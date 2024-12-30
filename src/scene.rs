use std::{cell::RefCell, collections::HashMap, rc::Weak};
use crate::*;
use std::rc::Rc;



#[derive(Clone,Default)]
pub struct UuidTags {
    tags: Rc<RefCell<HashMap<Uuid, Vec<String>>>>
}

impl UuidTags {
    pub fn get_tags(&self, uuid: &Uuid) -> Option<Vec<String>> {
        if let Some(tags) = self.tags.borrow().get(uuid) {
            Some(tags.clone())
        } else {
            None
        }
    }
    pub fn set_tags(&self, uuid: &Uuid, mut tag_vec: Vec<String>) {
        let mut all_tags = self.tags.borrow_mut();
        if let Some(tags) = all_tags.get_mut(uuid) {
            tags.drain(..);
            tags.append(&mut tag_vec);
        } else {
            all_tags.insert(*uuid, tag_vec);
        }
    }
    pub fn add_tag(&self, uuid: &Uuid, tag: String) {
        let mut all_tags = self.tags.borrow_mut();
        if let Some(tags) = all_tags.get_mut(uuid) {
            if !tags.contains(&tag) {
                tags.push(tag)
            }
        } else {
            all_tags.insert(*uuid, vec![tag]);
        }
    }
    pub fn has_tag(&self, uuid: &Uuid, tag: String) -> bool {
        if let Some(tags) = self.tags.borrow().get(uuid) {
            tags.contains(&tag)
        } else {
            false
        }
    }
}

impl element::ElementBehavior for UuidTags {}
impl element::ModuleBehavior for UuidTags {
    fn alias(&self) -> String {
        "uuid tags".to_string()
    }
    fn component(&self) -> &dyn std::any::Any {
        self
    }
}




#[derive(Clone)]
pub enum SceneEvent {
    SetCamera(Uuid),
    Instantiate(serde_json::Value),
    Delete(Uuid),
    JSONManager(Weak<JSONManager>)
}


use element::{ModuleTool,Element};
use event::Receiver;
pub struct Scene {
    pub mod_alias: HashMap<String,Uuid>,
    pub elements: HashMap<Uuid,Element>,
    camera_uuid: Uuid,


    sender: event::Sender<SceneEvent>,
    receiver: std::rc::Rc<Receiver<SceneEvent>>,


    json_manager: std::rc::Weak<JSONManager>,
}



impl Scene {
    pub fn new() -> Self {
        let s = event::Sender::new();
        let mut scene = Self {
            mod_alias: HashMap::new(),
            elements: HashMap::new(),
            camera_uuid: Uuid::new_v4(),

            receiver: s.new_receiver(),
            sender: s,

            json_manager: std::rc::Weak::new()
        };

        let sb_uuid = scene.add_element(Element::new_module(SceneBroadcastComponent::new(scene.sender.clone())));
        scene.mod_alias.insert("scene broadcast".to_string(), sb_uuid);


        let ut_uuid = scene.add_element(Element::new_module(UuidTags::default()));
        scene.mod_alias.insert("uuid tags".to_string(), ut_uuid);


        scene
    }
    pub fn module_tool(&self) -> ModuleTool {
        ModuleTool::new(self)
    }
    pub fn add_element(&mut self, element: Element) -> Uuid {
        let uuid = Uuid::new_v4();
        if let Element::Module(m) = &element {
            self.mod_alias.insert(m.borrow().alias(), uuid);
        }
        self.elements.insert(uuid, element);
        uuid
    }
    pub fn init_elements(&mut self) {
        //println!("\n--INIT ELEMENTS--");
        let mod_tools = self.module_tool();
        for (uuid, e) in &self.elements {
            e.init(*uuid, &mod_tools)
        }

        self.sender.send(SceneEvent::JSONManager(self.json_manager.clone()));
    }
    pub fn update_elements(&mut self, td:f32) {
        for e in self.receiver.poll() {
            match e {
                SceneEvent::SetCamera(uuid) => self.camera_uuid = uuid,
                SceneEvent::Instantiate(value) => {
                    println!("instant");
                    if let Some(json) = self.json_manager.upgrade() {
                        let uuid = self.add_element(json.create_element(&value));
                        self.elements.get(&uuid).unwrap().init(uuid, &self.module_tool());
                    }
                },
                SceneEvent::Delete(uuid) => {
                    self.elements.remove(&uuid);
                },
                SceneEvent::JSONManager(..) => ()
            }
        }
        //println!("\n--UPDATE ELEMENTS--");
        for (_uuid, e) in &self.elements {
            e.local_update(td);
        }
        //println!("  -post-");
        for (_uuid, e) in &self.elements {
            e.post_update();
        }

    }


    pub fn display(&self, sprite_sheet: &sprite::SpriteSheet) -> Vec<crate::Vertex> {
        let mut vertices = Vec::new();
        for (uuid, element) in &self.elements {
            if let Some(s) = element.sprite() {
                vertices.append(&mut sprite_sheet.vertices(s));
            }
        }
        vertices
    }


    pub fn camera_projection(&self, window_size: [u32;2]) -> ([[f32;3];3], [f32;2]) {
        if let Some(camera) = self.elements.get(&self.camera_uuid) {
            (camera.clip_matrix(window_size), camera.offset())
        } else {
            ([[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0f32]], [0.0,0.0])
        }
    }
}

pub struct SceneBroadcastComponent {
    sender: event::Sender<SceneEvent>
}
impl SceneBroadcastComponent {
    pub fn new(sender: event::Sender<SceneEvent>) -> Self {
        Self {
            sender: sender
        }
    }
}

impl element::ElementBehavior for SceneBroadcastComponent {}
impl element::ModuleBehavior for SceneBroadcastComponent {
    fn alias(&self) -> String {
        "sender broadcast".to_string()
    }
    fn component(&self) -> &dyn std::any::Any {
        &self.sender
    }
}



pub struct SceneManager {
    default_elements: HashMap<String, element::Element>,
    pub scenes: HashMap<String, Scene>,
    pub name: Option<&'static str>,

    pub json_manager: std::rc::Rc<JSONManager>,

    pub map_editor: bool
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            default_elements: HashMap::new(),
            scenes: HashMap::new(),
            name: None,

            json_manager: std::rc::Rc::new(JSONManager::new()),

            map_editor: false
        }
    }
    pub fn create_scene(&mut self, name: String, data: &serde_json::Value) {
        let mut scene = self.json_manager.create_scene(data);
        scene.json_manager = std::rc::Rc::downgrade(&self.json_manager);
        self.scenes.insert(name, scene);
    }
    pub fn current_scene(&mut self) -> Option<&mut Scene> {
        if let Some(name) = self.name {

            if let Some(s) = self.scenes.get_mut(name) {
                Some(s)
            } else { None }
        } else { None }
    }
    pub fn set_scene(&mut self, name: &'static str) -> Result<(),String> {
        if self.scenes.contains_key(name) {
            self.name = Some(name);
            Ok(())
        } else {
            self.name = None;
            Err("Scene not found".to_string())
        }
    }
}


pub struct JSONManager {
    pub element_names: HashMap<String, element::Element>
}

impl JSONManager {
    pub fn new() -> Self {
        Self {
            element_names: HashMap::new()
        }
    }
    pub fn create_element(&self, value: &serde_json::Value) -> Element {
        use serde_json::*;
        let mut e = Element::Null;
        if let Value::Object(element_data) = value {
            if let Some(Value::String(element_name)) = element_data.get("name") {
                if let Some(element) = self.element_names.get(element_name) {
                    e = element.load(&element_data);
                }
            }
        }
        e
    }
    pub fn create_scene(&self, data: &serde_json::Value) -> Scene {
        use serde_json::*;
        let mut scene = Scene::new();
        if let Value::Object(fields) = data {
            if let Some(Value::Array(elements)) = fields.get("elements") {
                for e in elements {
                    /*if let Value::Object(element_data) = e {
                        if let Some(Value::String(element_name)) = element_data.get("name") {
                            if let Some(element) = self.element_names.get(element_name) {
                                scene.add_element(element.load(&element_data));
                            }
                        }
                    }*/
                    match self.create_element(e) {
                        Element::Null => (),
                        x => {scene.add_element(x);}
                    }
                }
            }
        }
        scene
    }
}