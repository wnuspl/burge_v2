use crate::*;
use element::*;

#[serde(default)]
#[derive(Clone, Serialize, Deserialize)]
pub struct DefaultCamera {
	pub pos: [f32;2],
	pub scale: f32,
	aspect: f32
}

impl std::default::Default for DefaultCamera {
	fn default() -> Self {
		Self {
			pos: [0.0,0.0],
			scale: 20.0,
			aspect: 16./9.
		}
	}
}



impl ElementBehavior for DefaultCamera {
    fn init(&mut self, uuid: uuid::Uuid, mods: &ModuleTool) {
        mods.access("scene broadcast", |scene: &event::Sender<scene::SceneEvent> | {
            scene.send(scene::SceneEvent::SetCamera(uuid))
        });
	}
	fn load(&self, data: &serde_json::Map<String, serde_json::Value>) -> Element {
        if let Some(settings) = data.get("settings") {
            element::Element::new_gen(serde_json::from_value::<DefaultCamera>(settings.clone()).unwrap())
        } else {
            element::Element::new_gen(Self::default())
        }
	}
    fn clip_matrix(&self, window_size: [u32;2]) -> [[f32;3];3] {
		let aspect_ratio = window_size[0] as f32/window_size[1] as f32;
		let mut width = self.scale;
		let mut height = self.scale/self.aspect;
		if aspect_ratio > self.aspect { // more width
			width = height * aspect_ratio;
		} else if aspect_ratio < self.aspect {
			height = width / aspect_ratio;
		}
		[[2.0/width, 0.0, 0.0],
		[0.0, 2.0/height, 0.0],
		[0.0, 0.0, 1.0f32]]
	}
	fn offset(&self) -> [f32;2] {
		self.pos
	}
}