pub mod physics;
pub mod particles;
pub mod camera;
pub mod map_editor;




use crate::*;

#[serde(default)]
#[derive(Clone,Serialize,Deserialize)]
pub struct Block {
    name: String,
    pos: V2<f32>,
    shape: V2<f32>,
    solid: bool,
    sprite: super::sprite::Sprite,
    tags: Vec<String>
}

impl std::default::Default for Block {
    fn default() -> Self {
        Self {
            name: "block".to_string(),
            pos: V2::from([0.0,0.0]),
            shape: V2::from([1.0,1.0]),
            solid: false,
            sprite: sprite::Sprite::default(),
            tags: Vec::new()
        }
    }
}
impl Block {
    pub fn new_stretched(pos: V2<f32>, shape: V2<f32>, index: usize) -> Self {
        Self {
            pos: pos,
            shape: shape,
            sprite: sprite::Sprite { pos: pos, scale: shape, depth: 0.0, tex_indices: vec![vec![index]], next: None, flip: false },
            ..Default::default()
        }
    }
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.sprite.depth = depth;
        self
    }
    pub fn solid(mut self, is_solid: bool) -> Self {
        self.solid = is_solid;
        self
    }
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}


impl element::ElementBehavior for Block {
    
    fn init(&mut self, uuid: Uuid, mods: &element::ModuleTool) {
        if self.solid {
            mods.access("pom", |pom: &core::physics::POMComponent| {
                pom.new_sender().send(core::physics::PhysEvent::Static(
                    Some(uuid),
                    core::physics::PhysObj {
                        pos: self.pos,
                        shape: self.shape,
                        ..Default::default()
                    }
                ));
            });
        }
        mods.access("uuid tags", |ut: &scene::UuidTags | {
            ut.set_tags(&uuid, self.tags.clone());
        });
    }
    fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> element::Element {
        if let Some(s) = data.get("settings") {
            element::Element::new_gen(serde_json::from_value::<Block>(s.clone()).unwrap())
        } else {
            element::Element::Null
        }

        
        
        //element::Element::new_gen(serde_json::from_value::<Self>(serde_json::Value::Object(data.clone)).unwrap())
    }
    fn sprite(&self) -> Option<sprite::Sprite> {
        Some(self.sprite.clone())
    }
}