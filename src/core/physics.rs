use std::{cell::RefCell, rc::Rc};
use event::Sender;
use serde::{Deserialize,Serialize};

use crate::*;

#[derive(Copy,Clone, Serialize, Deserialize)]
pub struct PhysObjSettings {
    pub gravity_strength: f32,
    pub drag: V2<f32> // applied every second to x velocity towards 0
}
impl std::default::Default for PhysObjSettings {
    fn default() -> Self {
        Self {
            gravity_strength: -8.0,
            drag: [0.0,0.0].into()
        }
    }
}

#[serde(default)]
#[derive(Clone,Copy, Serialize, Deserialize)]
pub struct PhysObj {
    pub pos: V2<f32>,
    pub shape: V2<f32>,
    pub velocity: V2<f32>,
    pub settings: PhysObjSettings
}

impl std::default::Default for PhysObj {
    fn default() -> Self {
        Self {
            pos: V2::from([0.0,0.0]),
            shape: V2::from([1.0,1.0]),
            velocity: V2::from([0.0,0.0]),
            settings: PhysObjSettings::default()
        }
    }
}

impl PhysObj {
    pub fn center(&self) -> V2<f32> {
        V2 {
            x: self.pos.x + (self.shape.x/2.0),
            y: self.pos.y + (self.shape.y/2.0)
        }
    }
    pub fn intersects(&self, other: &PhysObj, delta: V2<f32>) -> bool {
        let this_left = self.pos.x;
        let this_right = self.pos.x+self.shape.x;
        let this_bottom = self.pos.y;
        let this_top = self.pos.y+self.shape.y;

        let other_left = other.pos.x;
        let other_right = other.pos.x+other.shape.x;
        let other_bottom = other.pos.y;
        let other_top = other.pos.y+other.shape.y;

        !(this_left + delta.x > other_right || this_right + delta.x < other_left || this_bottom + delta.y > other_top || this_top + delta.y < other_bottom)
    }
    pub fn nearest_delta(&self, other: &PhysObj) -> V2<f32> {
        let left = other.pos.x - (self.pos.x+self.shape.x);
        let right = (other.pos.x + other.shape.x) - self.pos.x;
        let down = other.pos.y - (self.pos.y+self.shape.y);
        let up = (other.pos.y + other.shape.y) - self.pos.y;

        let horizontal = if left.abs() > right.abs() {
            right
        } else {
            left
        };

        let vertical = if down.abs() > up.abs() {
            up
        } else {
            down
        };

        if horizontal.abs() < vertical.abs() {
            [horizontal, 0.0].into()
        } else {
            [0.0, vertical].into()
        }
    }
    pub fn individual_update(&self, td:f32) -> Vec<PhysEvent> {
        let mut eq = Vec::new();
        eq.push(PhysEvent::ModPos(self.velocity*td));

        if self.settings.gravity_strength != 0.0 {
            eq.push(PhysEvent::ModVelocity([0.0,self.settings.gravity_strength*td].into()));
        }


        /*if self.velocity.x.abs() > self.settings.drag.x*td {
            eq.push(PhysEvent::ModVelocity([self.settings.drag.x*self.velocity.x.signum()*td*-1.0,0.0].into()));
        } else {
            eq.push(PhysEvent::SetVelocity([Some(0.0),None].into()));
        }

        if self.velocity.y.abs() > self.settings.drag.y*td {
            eq.push(PhysEvent::ModVelocity([0.0,self.settings.drag.y*self.velocity.y.signum()*td*-1.0].into()));
        } else {
            eq.push(PhysEvent::SetVelocity([None,Some(0.0)].into()));
        }*/

        eq
    }

    pub fn update(&mut self, events: Vec<PhysEvent>) -> Vec<PhysEvent> {
        let mut leftover = Vec::new();
        for e in events {
            match e {
                PhysEvent::ModPos(p) => self.pos = self.pos + p,
                PhysEvent::ModVelocity(v) => self.velocity = self.velocity + v,
                PhysEvent::ScalePos(p) => {self.pos.x *= p.x; self.pos.y *= p.y},
                PhysEvent::ScaleVelocity(v) => {self.velocity.x *= v.x; self.velocity.y *= v.y},

                PhysEvent::SetVelocity(v) => {
                    if let Some(x) = v.x {
                        self.velocity.x = x;
                    }
                    if let Some(y) = v.y {
                        self.velocity.y = y;
                    }
                },


                _ => leftover.push(e)
            }
        }
        leftover
    }
}

#[derive(Clone,Copy)]
pub enum PhysEvent {
    Collision(Option<Uuid>, PhysObj),
    Static(Option<Uuid>, PhysObj),
    Dynamic(Uuid, PhysObj),
    
    ScalePos(V2<f32>),
    ModPos(V2<f32>),

    ScaleVelocity(V2<f32>),
    ModVelocity(V2<f32>),
    SetVelocity(V2<Option<f32>>)
}




pub struct PhysObjManager {
    sender: event::Sender<PhysEvent>,

    receiver: Rc<event::Receiver<PhysEvent>>,
    facet: event::Sender<PhysEvent>,

    statics: Vec<(Option<Uuid>,PhysObj)>,



    component: POMComponent,


    send_queue: Vec<(Uuid, Vec<PhysEvent>)>
}


#[derive(Clone)]
pub struct POMComponent {
    sender: event::Sender<PhysEvent,event::Locked>,
    facet: event::Sender<PhysEvent>
}
impl POMComponent {
    pub fn default() -> Self {
        Self {
            sender: Sender::new(),
            facet: Sender::new()
        }
    }
    pub fn new_routed_receiver(&self, uuid: Option<Uuid>) -> (Uuid, Rc<event::Receiver<PhysEvent>>) {
        self.sender.new_routed_receiver(uuid)
    }
    pub fn new_sender(&self) -> event::Sender<PhysEvent> {
        self.facet.clone()
    }
}


impl PhysObjManager {
    pub fn new() -> Self {
        let o = event::Sender::new();
        let i = event::Sender::new();

        Self {
            sender: o.clone(),

            receiver: i.new_receiver(),
            facet: i.clone(),

            statics: Vec::new(),


            component: POMComponent {
                sender: o.lock(),
                facet: i
            },

            send_queue: Vec::new()
        }
    }
}
impl element::ElementBehavior for PhysObjManager {
    fn local_update(&mut self, td: f32) {
        let mut dynamics = Vec::new();
        for e in self.receiver.poll() {
            //println!("Received Event");
            match e {
                PhysEvent::Dynamic(uuid, po) => dynamics.push((uuid,po)),
                PhysEvent::Static(uuid, po) => self.statics.push((uuid, po)),
                _ => ()
            }
            
        }



        for (uuid, d) in &dynamics {
            let mut event_queue = Vec::new();

            event_queue.append(&mut d.individual_update(td));


            for (s_uuid, s) in &self.statics {
                if d.intersects(s, d.velocity*td) {
                    event_queue.push(PhysEvent::Collision(*s_uuid, s.clone()));

                    /*let delta = d.nearest_delta(s);

                    if delta.y == 0.0 {
                        if delta.x.signum() != d.velocity.x.signum() {
                            event_queue.push(PhysEvent::SetVelocity([Some(0.0),None].into()));
                        }
                    } else {
                        if delta.y.signum() != d.velocity.y.signum() {
                            event_queue.push(PhysEvent::SetVelocity([None,Some(0.0)].into()));
                        }
                    }*/
                }
            }

            for (other_uuid, other_d) in &dynamics {
                if uuid != other_uuid {
                    if d.intersects(other_d, d.velocity*td) {
                        event_queue.push(PhysEvent::Collision(Some(*other_uuid), other_d.clone()));
                    }
                }
            }


            self.send_queue.push((*uuid, event_queue))
        }
    }
    fn post_update(&mut self) {
        for (uuid, events) in self.send_queue.drain(..) {
            for e in events {
                self.sender.route(uuid, e)
            }
        }
    }
    fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> element::Element {
        element::Element::Module(RefCell::new(Box::new(Self::new())))
    }
}

impl element::ModuleBehavior for PhysObjManager {
    fn alias(&self) -> String {
        "pom".to_string()
    }
    fn component(&self) -> &dyn std::any::Any {
        &self.component
    }
}