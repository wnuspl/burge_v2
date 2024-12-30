use std::{ops::Deref, path, time::{self, Duration, Instant}};
use crate::{element, event::Sender, scene::SceneManager, Vertex};
use std::path::Path;
use glium::{glutin::{dpi::PhysicalSize, event::{ElementState, KeyboardInput}, event_loop::ControlFlow}, implement_vertex, texture, Surface, Texture2d};
use std::collections::HashMap;

use std::rc::Rc;
use std::cell::RefCell;
use crate::sprite;


#[derive(Clone, Copy)]
pub enum InputEvent {
    KeyDown(u32),
    KeyUp(u32)
}
#[derive(Clone)]
pub struct InputManager {
    sender: Sender<InputEvent, crate::event::Locked>
}

impl element::ModuleBehavior for InputManager {
    fn alias(&self) -> String {
        "input".to_string()
    }
    fn component(&self) -> &dyn std::any::Any {
        &self.sender
    }
}
impl element::ElementBehavior for InputManager {
    fn load(&self, data: &serde_json::Map<String,serde_json::Value>) -> element::Element {
        element::Element::new_module(self.clone())
    }
}


pub struct Instance {
    pub scene_manager: SceneManager,

    pub bg_color: (f32,f32,f32,f32),
    pub aspect_ratio: f32,
    pub fullscreen: bool,
    pub sprite_sheet: sprite::SpriteSheet,
    map_editor_ss: sprite::SpriteSheet,
    pub ss_path: &'static str,


    input: Sender<InputEvent>,
    input_manager: InputManager
}
impl std::default::Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}

impl Instance {
    pub fn new() -> Self {
        let i = Sender::new();
        Self {
            scene_manager: SceneManager::new(),
            bg_color: (0.0,0.0,0.0,1.0),
            aspect_ratio: 16.0/9.0,

            fullscreen: false,
            sprite_sheet: sprite::SpriteSheet::new((8,8)),
            map_editor_ss: sprite::SpriteSheet::new((8,8)),
            ss_path: "",

            input_manager: InputManager {
                sender: i.lock()
            },
            input: i
        }
    }
    pub fn scene_manager(&mut self) -> &mut SceneManager {
        &mut self.scene_manager
    }
    pub fn input(&self) -> InputManager{
        self.input_manager.clone()
    }
    pub fn start(mut self) {
        use glium::{glutin, Surface};
        let event_loop = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new().with_inner_size(PhysicalSize { width: 800., height: 800. / self.aspect_ratio }).with_title("burge");


        let cb = glutin::ContextBuilder::new().with_vsync(true).with_srgb(true);
        let display = glium::Display::new(wb, cb,&event_loop).unwrap();

        

    



        
        let vertex_shader = include_str!("shaders/vertex.glsl");
        let fragment_shader= include_str!("shaders/fragment.glsl");
        let program = glium::Program::new(
            &display,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        ).unwrap();


        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        use glium::uniforms::*;
        let sampler_behavior = glium::uniforms::SamplerBehavior {
            magnify_filter: MagnifySamplerFilter::Nearest,
            minify_filter: MinifySamplerFilter::Nearest,
            ..Default::default()
        };
        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            blend: glium::draw_parameters::Blend::alpha_blending(),
            .. Default::default()
        };
        let mut window_size: PhysicalSize<u32> = PhysicalSize {width: 800, height: (800. / self.aspect_ratio) as u32};


        let image = image::open(self.ss_path).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
        
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::Texture2d::new(&display, image).unwrap();
        self.sprite_sheet.tex = Some(texture);

        /*self.map_editor_ss.tex = {
            let image = image::open("src/map_edit_sprites.png").unwrap().to_rgba8();
            let image_dimensions = image.dimensions();
            
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
            let texture = glium::Texture2d::new(&display, image).unwrap();
            Some(texture)
        };*/

        for (_, scene) in self.scene_manager.scenes.iter_mut() {
            scene.init_elements();
        }

        //let static_buffer = glium::VertexBuffer::new(&display, &self.scene_manager.current_scene().static_sprites).unwrap();


        let mut time_delta: f32 = 0.0;
        //std::thread::sleep(Duration::new(5,100_000));
        let _ = event_loop.run(move |event, _, control_flow | {
            std::thread::sleep(Duration::new(0,100_000));
            
            let start_time = Instant::now();
            *control_flow = ControlFlow::Poll;
            //*control_flow = ControlFlow::WaitUntil(Instant::now()+Duration::from_millis((unsafe { 1.0 / crate::FRAME_RATE }*1000.0 - time_delta*1000.0) as u64));
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        //self.scene_controller.save();
                        *control_flow = ControlFlow::Exit;
                    },
                    glutin::event::WindowEvent::Resized(size) => {
                        window_size = size;
                    },
                    glutin::event::WindowEvent::KeyboardInput { input: KeyboardInput {scancode, state, ..}, .. } => {
                        
                        match state {
                            ElementState::Pressed => self.input.send(InputEvent::KeyDown(scancode)),
                            ElementState::Released => self.input.send(InputEvent::KeyUp(scancode)),
                        }
                    },
                    _ => ()
                },
                
                glutin::event::Event::RedrawRequested(_) => {
                    
                    let mut target = display.draw();

                    target.clear_color_and_depth(self.bg_color, 1.0);

                    

                    if let Some(scene) = self.scene_manager.current_scene() {

                        
                        let vertices = scene.display(&self.sprite_sheet);

                        let vertex_buffer: glium::VertexBuffer<Vertex> = glium::VertexBuffer::new(&display, &vertices).unwrap();

                        
                        
                        let (ortho_mat, translation) = scene.camera_projection(window_size.into());
                        let uniforms = glium::uniform! {
                            ortho_mat: ortho_mat,
                            translation: translation,
                            tex: glium::uniforms::Sampler(self.sprite_sheet.tex.as_ref().unwrap(), sampler_behavior)
                        };
                
                        target.draw(&vertex_buffer, &indices, &program, &uniforms, &draw_parameters).unwrap();
                        
                        //target.draw(&static_buffer, &indices, &program, &uniforms, &draw_parameters).unwrap();
                        if self.scene_manager.map_editor {
                            
                        }
                        
                        
                    }
                    target.finish().unwrap();
                },
                glutin::event::Event::MainEventsCleared => {
                    
                }
                _ => ()
            }
            if let Some(scene) = self.scene_manager.current_scene() {
                scene.update_elements(time_delta);
            }
            display.gl_window().window().request_redraw();

            std::thread::sleep(Duration::new(0,500_000));

            let end_time = Instant::now();
            time_delta = (end_time-start_time).as_secs_f32();
        });
    }
}