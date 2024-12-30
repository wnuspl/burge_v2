# burge

## Basic Uniform Rust Game Engine

A game engine able to handle simple 2d games in Rust. Based on the idea of uniformity, all elements follow a simple format that can easily be expanded upon while still conforming to the base structure.

Features:
- Scene management
- Saving and loading game data (as JSON)
- Flexible event system
- Sprite rendering
- Post-processing
- Physics system


## Getting Started

An instance is the basis of any game. It will handle the game loop, inputs, updates, and rendering.
```rust
use burge::*; 

fn main() {
    let mut instance = instance::Instance::new();

    instance.run();
}
```

To add an element to the game, you must also provide instructions on how to load it from JSON.

```rust
use burge::*;
use element::*;
use serde_json::{Value,Map};

#[derive(Default,Clone)]
struct Player;


// All elements must implement ElementBase
// All methods of ElementBase are optional, but many should be implemented often
impl ElementBase for Player { 
    fn load(&self, data: Map<String, Value>) -> Element {
        Element::Element(Box::new(self.clone))
    }
}




fn main() {
    let mut instance = instance::Instance::new();

    // A JSONManager will store a default version of each element, as well as a string identifier for it
    // The load method will be called on the default element corresponding to the 'name' field, and that new element will be added to the scene
    instance.scene_controller().set_json_manager({
        let mut jm = scene::JSONManager::new();
        let default_player = Element::Element(Box::new(Player::default()));
        jm.elements.insert("player", default_player);
        jm
    });

    let data: serde_json::Value = serde_json::json!("r#
    {
        'name': 'main',
        'components': [],
        'elements': [
            { 'name': 'player' }
        ]
    }
    "#);

    instance.scene_controller().create_scene(data);

    instance.run();
}

```
Although tedious to start, this gives the user control of how every element is created and allows for more complicated implementations of elements to be handled the same way.

To add more functionality to our player, lets add a position value that increases every frame.
```rust
use burge::*;
use element::*;
use serde_json::{Value,Map};

#[derive(Default,Clone)]
struct Player {
    pos: [f32;2]
}

impl ElementBase for Player { 
    fn local_update(&mut self, td: f32) {
        self.pos[0] += 0.1*td;
    }
    fn load(&self, data: Map<String, Value>) -> Element {
        let mut default = self.clone();

        // Allow pos to be loaded from JSON
        if let Some(pos) = data.get("pos") {
            // try_vec2! will return Option<[f32;2]> from serde_json::Value
            default.pos = try_vec2!(pos).unwrap();

        }

        Element::Element(Box::new(default))
    }
}
```
