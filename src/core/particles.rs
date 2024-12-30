use crate::*;
use self::core::physics::PhysObjSettings;
use self::core::physics::PhysObj;
use std::rc::Rc;
use serde::{Serialize, Deserialize};

#[derive(Clone,Serialize,Deserialize,Default)]
pub struct ParticleNormal;

#[serde(default)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Particle<Type:ParticleBehavior> {
    #[serde(skip)]
    pub timer: f32,

    pub max_life: f32,

    pub phys_obj: core::physics::PhysObj,

    #[serde(skip)]
    pub data: Box<Type>,

    #[serde(skip)]
    pub phantom: std::marker::PhantomData<Type>
}

pub trait ParticleBehavior: Clone + Default {
    fn over_lifetime(particle: &mut Particle<Self>, lifetime:f32) {}
    fn init(particle: &mut Particle<Self>) {}
}
impl ParticleBehavior for ParticleNormal {}


impl<Type:ParticleBehavior> std::default::Default for Particle<Type> {
    fn default() -> Self {
        Self {
            timer: 3.0,
            max_life: 3.0,
            data: Box::new(Type::default()),
            phys_obj: core::physics::PhysObj {
                pos: [0.0,0.0].into(),
                shape: [0.1,0.1].into(),
                velocity: [0.0,0.0].into(),
                settings: PhysObjSettings {
                    gravity_strength: -0.5,
                    drag: [0.25,0.25].into()
                }
            },

            phantom: std::marker::PhantomData
        }
    }
}

#[derive(Clone)]
pub enum ParticleEvent {
    Emit
}

#[serde(default)]
#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleEmitterSettings<Type:ParticleBehavior> {
    pub base: serde_json::Value,

    #[serde(skip)]
    pub phantom: std::marker::PhantomData<Type>,
    pub origin: V2<f32>,
    pub angle: f32, //radians
    pub spread: f32,
    pub count: u32,
    pub velocity: V2<f32>,
    pub sprite_index: usize,
    pub sprite_depth: f32,

    pub spread_random: f32,
    pub count_random: u32,
    pub velocity_random: V2<f32>,
}

impl<Type:ParticleBehavior> std::default::Default for ParticleEmitterSettings<Type> {
    fn default() -> Self {
        Self {
            base: serde_json::to_value(Particle::<Type>::default()).unwrap(),
            phantom: std::marker::PhantomData,
            angle: 0.0,
            origin: [0.0,0.0].into(),
            spread: 3.14159*2.0,
            count: 20,
            velocity: [1.0,1.0].into(),


            spread_random: 0.5,
            count_random: 0,
            velocity_random: [0.5,0.5].into(),
            sprite_index: 0,
            sprite_depth: 0.0
        }
    }
}


#[derive(Clone)]
pub struct ParticleEmitter<Type:ParticleBehavior = ParticleNormal> {
    pub data: ParticleEmitterSettings<Type>,
    particles: Vec<Particle<Type>>,
    sender: event::Sender<ParticleEvent>,
    receiver: Rc<event::Receiver<ParticleEvent>>,
    id: String
}

impl<Type:ParticleBehavior> std::default::Default for ParticleEmitter<Type> {
    fn default() -> Self {
        let mut s = event::Sender::new();
        Self {
            data: ParticleEmitterSettings::default(),
            particles: Vec::new(),
            receiver: s.new_receiver(),
            sender: s,
            id: "particle".to_string()
        }
    }
}


impl<Type:ParticleBehavior> ParticleEmitter<Type> {
    pub fn emit(&mut self) {
        println!("Particling");
        let mut particles = Vec::new();
        let count = {
            let r = rand::random::<f32>() - 0.5;
            self.data.count - (r * self.data.count_random as f32) as u32
        };
        let mut p_base: Particle<Type> = serde_json::from_value(self.data.base.clone()).unwrap();
        p_base.phys_obj.pos = self.data.origin;
        let spread_interval = self.data.spread/(count) as f32;
        for i in 1..count+1 {

            let spread_r = {
                let r = rand::random::<f32>() - 0.5;
                r*self.data.spread_random*spread_interval
            };

            let this_angle = spread_interval*i as f32 + self.data.angle + spread_r;

            let velocity_r: V2<f32> = {
                let rx = rand::random::<f32>() - 0.5;
                let ry = rand::random::<f32>() - 0.5;
                [rx*self.data.velocity.x*self.data.velocity_random.x, ry*self.data.velocity.y*self.data.velocity_random.x].into()
            };
            let velocity = V2 {
                x: (self.data.velocity.x * this_angle.cos()) + velocity_r.x,
                y: (self.data.velocity.y * this_angle.sin()) + velocity_r.y
            };
            
            let mut p = p_base.clone();
            p.phys_obj.velocity = velocity;
            Type::init(&mut p);
            particles.push(p)
        }


        self.particles.append(&mut particles);
    }
}



impl<Type:ParticleBehavior+ 'static> element::ElementBehavior for ParticleEmitter<Type> {
    fn local_update(&mut self, td: f32) {
        for e in self.receiver.poll() {
            match e {
                ParticleEvent::Emit => self.emit()
            }
        }
        let mut remove_idx = Vec::new();
        for (idx, p) in &mut self.particles.iter_mut().enumerate() {
            let updates = p.phys_obj.individual_update(td);
            p.phys_obj.update(updates);
            Type::over_lifetime(p, 1.0 - p.timer/p.max_life);

            if p.timer < 0.0 {
                remove_idx.push(idx);
            } else {
                p.timer -= td;
                //p.phys_obj.shape = [(p.lifetime/3.0)*0.2,(p.lifetime/3.0)*0.2].into();
            }
        }

        let mut offset = 0;
        for i in remove_idx {
            self.particles.remove(i-offset);
            offset += 1;
        }
    }
    fn sprite(&self) -> Option<sprite::Sprite> {
        let mut base = sprite::Sprite::empty();

        let mut sprite = sprite::Sprite::single(self.data.sprite_index);

        for p in &self.particles {
            base.next(sprite.clone().with_pos(p.phys_obj.pos).with_scale(p.phys_obj.shape).with_depth(self.data.sprite_depth));
        }
        Some(base)
    }
    fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> element::Element {
        
        let mut s: ParticleEmitter<Type> = Self::default();

        if let Some(settings) = data.get("settings") {
            s.data = serde_json::from_value(settings.clone()).unwrap();
        }


        s.emit();
        element::Element::new_module(s)
    }
}

impl<Type:ParticleBehavior + 'static> element::ModuleBehavior for ParticleEmitter<Type> {
    fn alias(&self) -> String {
        self.id.clone()
    }
    fn component(&self) -> &dyn std::any::Any {
        &self.sender
    }
}