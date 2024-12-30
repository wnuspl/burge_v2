use std::collections::HashMap;
use std::cell::RefCell;
use crate::*;

pub trait ElementBehavior {
    fn init(&mut self, uuid: Uuid, mods: &ModuleTool) {}
	fn local_update(&mut self, td: f32) {}
	fn post_update(&mut self) {}
	
	fn save(&self) -> serde_json::Value { serde_json::Value::Null }
	fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> Element { Element::Null }

    fn clip_matrix(&self, window_size: [u32;2]) -> [[f32;3];3] {
		[[1.0, 0.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 0.0, 1.0]]
	}
	fn offset(&self) -> [f32;2] { [0.0,0.0] }


    fn sprite(&self) -> Option<sprite::Sprite> { None }
}


pub enum Element {
    Gen(RefCell<Box<dyn ElementBehavior>>),
    Group(RefCell<Vec<Element>>),
    Module(RefCell<Box<dyn ModuleBehavior>>),
    Null
}



impl Element {
    pub fn new_gen<T: ElementBehavior + 'static>(e: T) -> Element {
        Element::Gen(RefCell::new(Box::new(e)))
    }
    pub fn new_module<T: ModuleBehavior + 'static>(m: T) -> Element {
        Element::Module(RefCell::new(Box::new(m)))
    }
    pub fn init(&self, uuid: Uuid, mods: &ModuleTool) {
        match self {
            Element::Gen(e) => e.borrow_mut().init(uuid, mods),
            Element::Module(m) => m.borrow_mut().init(uuid, mods),
            Element::Group(g) => for e in g.borrow_mut().iter() { e.init(uuid, mods); },
            Element::Null => ()
        }
    }
    pub fn local_update(&self, td:f32) {
        match self {
            Element::Gen(e) => e.borrow_mut().local_update(td),
            Element::Module(m) => m.borrow_mut().local_update(td),
            Element::Group(g) => for e in g.borrow_mut().iter() { e.local_update(td); },
            Element::Null => ()
        }
    }
    pub fn post_update(&self) {
        match self {
            Element::Gen(e) => e.borrow_mut().post_update(),
            Element::Module(m) => m.borrow_mut().post_update(),
            Element::Group(g) => for e in g.borrow_mut().iter() { e.post_update(); },
            Element::Null => ()
        }
    }
    pub fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> Element {
        match self {
            Element::Gen(e) => e.borrow_mut().load(data),
            Element::Module(m) => m.borrow_mut().load(data),
            Element::Group(g) => Self::Group({
                let mut v = Vec::new();
                for e in g.borrow_mut().iter() { v.push(e.load(&data)); }
                v.into()
            }),
            Element::Null => Element::Null
        }
    }
    pub fn save(&self) -> serde_json::Value {
        match self {
            Element::Gen(e) => e.borrow_mut().save(),
            Element::Module(m) => m.borrow_mut().save(),
            Element::Group(g) => serde_json::Value::Null,
            Element::Null => serde_json::Value::Null
        }
    }



    pub fn clip_matrix(&self, window_size: [u32;2]) -> [[f32;3];3] {
		match self {
            element::Element::Gen(e) => e.borrow_mut().clip_matrix(window_size),
            element::Element::Module(m) => m.borrow_mut().clip_matrix(window_size),
            element::Element::Group(g) => 
                [[1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0]],
			element::Element::Null => [[1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0]]
		}
	}
    pub fn offset(&self) -> [f32;2] {
		match self {
            element::Element::Gen(e) => e.borrow_mut().offset(),
            element::Element::Module(m) => m.borrow_mut().offset(),
            element::Element::Group(g) => [0.0,0.0],
			element::Element::Null => [0.0,0.0]
		}
	}




    pub fn sprite(&self) -> Option<sprite::Sprite> {
		match self {
            element::Element::Gen(e) => e.borrow_mut().sprite(),
            element::Element::Module(m) => m.borrow_mut().sprite(),
            element::Element::Group(g) => {
                let mut s = sprite::Sprite::empty();
                for e in g.borrow_mut().iter() {
                    if let Some(sprite) = e.sprite() {
                        s.next(sprite);
                    }
                }
                Some(s)
            },
			element::Element::Null => None
		}
	}
}




pub trait ModuleBehavior: ElementBehavior {
    fn alias(&self) -> String;
    fn component(&self) -> &dyn std::any::Any;
}


pub struct ModuleTool<'a> {
    scene: &'a scene::Scene
}

impl<'a> ModuleTool<'a> {
    pub fn new(scene: &'a scene::Scene) -> Self {
        Self {
            scene: scene
        }
    }
    pub fn access<T: 'static>(&self, alias: &str,  mut function: impl FnMut(&T)) {
        if let Some(uuid) = self.scene.mod_alias.get(alias) {
            if let Some(element) = self.scene.elements.get(uuid) {
                if let Element::Module(m) = element {
                    // Have module
                    let m = m.borrow();
                    let component = m.component();
                    if let Some(casted) = component.downcast_ref::<T>() {
                        function(casted)
                    }
                }
            }
        }
    }
}






