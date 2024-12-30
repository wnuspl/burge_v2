use super::*;

use super::{camera::DefaultCamera, element::{self, Element}, instance::InputEvent, scene::JSONManager};
use std::collections::HashMap;
use std::{iter::Map, rc::Rc};


enum MapEditorState {
    Pan,
    Zoom,
    ElementSelected {
        //sliders: HashMap<String, Vec<[f32;2]>>
    }
}


pub struct MapEditor {
    scene: event::Sender<scene::SceneEvent>,
    input_receiver: Rc<event::Receiver<instance::InputEvent>>,

    placed_elements: Vec<Element>,

    camera: DefaultCamera,
    scene_receiver: Rc<event::Receiver<scene::SceneEvent>>,
    json_manager: std::rc::Weak<JSONManager>,
    order: Vec<String>,
    idx: usize,

    current_element: Option<serde_json::Map<String,serde_json::Value>>,



    state: MapEditorState
}

impl MapEditor {
    pub fn new() -> Self {
        Self {
            scene: event::Sender::new(),
            input_receiver: event::Receiver::new(),

            placed_elements: Vec::new(),
            current_element: None,

            scene_receiver: event::Receiver::new(),
            json_manager: std::rc::Weak::new(),
            order: Vec::new(),
            idx: 0,

            camera: DefaultCamera::default(),
            
            state: MapEditorState::Pan
        }
    }
}

const UP: u32 = 72;
const LEFT: u32 = 75;
const DOWN: u32 = 80;
const RIGHT: u32 = 77;

const H: u32 = 35;
const Z: u32 = 44;
const R: u32 = 19;
const E: u32  = 18;
const C: u32 = 46;


const W: u32 = 17;
const A: u32 = 30;
const S: u32 = 31;
const D: u32 = 32;

impl element::ElementBehavior for MapEditor {
    fn clip_matrix(&self, window_size: [u32;2]) -> [[f32;3];3] {
        self.camera.clip_matrix(window_size)
    }
    fn offset(&self) -> [f32;2] {
        self.camera.pos
    }
    fn init(&mut self, uuid: uuid::Uuid, mods: &element::ModuleTool) {
        mods.access("scene broadcast", |scene: &event::Sender<scene::SceneEvent> | {
            scene.send(scene::SceneEvent::SetCamera(uuid));
            self.scene = scene.clone();
            self.scene_receiver = scene.new_receiver();
        }); 
        mods.access("input", |input: &event::Sender<instance::InputEvent, event::Locked>| {
            self.input_receiver = input.new_receiver();
        });
    }
    fn local_update(&mut self, td: f32) {
        for e in self.scene_receiver.poll() {
            match e {
                scene::SceneEvent::JSONManager(json) => {
                    self.json_manager = json;
                    if let Some(j) = self.json_manager.upgrade() {
                        self.order = j.element_names.keys().map(|s|{s.clone()}).collect();
                    }
                },
                _ => ()
            }
        }
        let mut key_down = Vec::new();
        for e in self.input_receiver.poll() {
            match e {
                InputEvent::KeyDown(scancode) => match scancode {
                    Z => self.state = MapEditorState::Zoom,
                    H => self.state = MapEditorState::Pan,
                    E => self.state = MapEditorState::ElementSelected {},
                    57 => println!("{}",serde_json::to_string_pretty(&self.save()).unwrap()),
                    _ => key_down.push(scancode),
                },
                InputEvent::KeyUp(..) => ()
            }
        }
        match self.state {
            MapEditorState::Pan => {
                let speed = 0.5;
                for k in key_down {
                    match k {
                        UP => self.camera.pos[1] += speed,
                        DOWN => self.camera.pos[1] += -speed,
                        LEFT => self.camera.pos[0] += -speed,
                        RIGHT => self.camera.pos[0] += speed,
                        _ => ()
                    }
                }
            },
            MapEditorState::Zoom => {
                let speed = 0.5;
                for k in key_down {
                    match k {
                        UP => self.camera.scale += -speed,
                        DOWN => self.camera.scale += speed,
                        _ => ()
                    }
                }
            },
            MapEditorState::ElementSelected {} => {
                let speed = 0.1;
                if let Some(e) = &mut self.current_element {
                    if let Some(serde_json::Value::Array(pos)) = e.get("pos") {
                        let mut pos_v2 = V2 {
                            x: pos[0].as_number().unwrap().as_f64().unwrap() as f32,
                            y: pos[1].as_number().unwrap().as_f64().unwrap() as f32,
                        };
                        for k in &key_down {
                            match *k {
                                UP => pos_v2.y += speed,
                                DOWN => pos_v2.y += -speed,
                                LEFT => pos_v2.x += -speed,
                                RIGHT => pos_v2.x += speed,
                                _ => ()
                            }
                        }
                        e["pos"] = serde_json::to_value([pos_v2.x,pos_v2.y]).unwrap();
                    } else if let Some(serde_json::Value::Object(settings)) = e.get("settings") {
                        if let Some(serde_json::Value::Array(pos)) = settings.get("pos") {
                            let mut pos_v2 = V2 {
                                x: pos[0].as_number().unwrap().as_f64().unwrap() as f32,
                                y: pos[1].as_number().unwrap().as_f64().unwrap() as f32,
                            };
                            for k in &key_down {
                                match *k {
                                    UP => pos_v2.y += speed,
                                    DOWN => pos_v2.y += -speed,
                                    LEFT => pos_v2.x += -speed,
                                    RIGHT => pos_v2.x += speed,
                                    _ => ()
                                }
                            }
                            e["pos"] = serde_json::to_value([pos_v2.x,pos_v2.y]).unwrap();
                        }
                    }



                    if let Some(serde_json::Value::Array(pos)) = e.get("shape") {
                        let mut pos_v2 = V2 {
                            x: pos[0].as_number().unwrap().as_f64().unwrap() as f32,
                            y: pos[1].as_number().unwrap().as_f64().unwrap() as f32,
                        };
                        for k in &key_down {
                            match *k {
                                W => pos_v2.y += speed,
                                S => pos_v2.y += -speed,
                                A => pos_v2.x += -speed,
                                D => pos_v2.x += speed,
                                _ => ()
                            }
                        }
                        e["shape"] = serde_json::to_value([pos_v2.x,pos_v2.y]).unwrap();
                    }
                }


                for k in &key_down {
                    match *k {
                        R => {
                            if let Some(json) = self.json_manager.upgrade() {
                                println!("{:?}", self.order);
                                self.idx += 1;
                                if self.idx >= self.order.len() {
                                    self.idx = 0;
                                }
                                let name = &self.order[self.idx];
                                if let Some(e) = json.element_names.get(name) {
                                    if let serde_json::Value::Object(map) = e.save() {
                                        self.current_element = Some(map);
                                    }
                                }
                            }
                        },
                        C => {
                            if let Some(json) = self.json_manager.upgrade() {
                                if let Some(e) =& self.current_element {
                                    self.placed_elements.push(json.create_element(&serde_json::Value::Object(e.clone())));
                                }
                            }
                        }
                        _ => ()
                    }
                }
            }
            _ => ()



        }

        
        
    }



    fn save(&self) -> serde_json::Value {
        let mut arr = Vec::new();

        for e in &self.placed_elements {
            arr.push(e.save());
        }

        /*let mut scene = serde_json::Map::new();
        scene.insert("elements".to_string(), serde_json::Value::Array(arr));
        serde_json::Value::Object(scene)*/
        serde_json::Value::Array(arr)
    }
    fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> Element {
        Element::new_gen(Self::new())
    }
    fn sprite(&self) -> Option<super::sprite::Sprite> {
        let mut sprite = sprite::Sprite::empty();
        for e in &self.placed_elements {
            if let Some(s) = e.sprite() {
                sprite.next(s);
            }
        }
        if let Some(e) = &self.current_element {
            if let Some(json) = self.json_manager.upgrade() {
                sprite.next(json.create_element(&serde_json::Value::Object(e.clone())).sprite().unwrap())
            }
        }

        Some(sprite)
    }
}