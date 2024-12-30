use crate::*;
use serde::{Deserialize,Serialize};

#[derive(Clone,Serialize,Deserialize,Default)]
pub struct Sprite {
    pub pos: V2<f32>,
    pub scale: V2<f32>,
    pub depth: f32,
    pub tex_indices: Vec<Vec<usize>>,
    pub next: Option<Box<Self>>,
    pub flip: bool
}

impl Sprite {
    pub fn single(index: usize) -> Sprite {
        Self {
            pos: [0.0,0.0].into(),
            scale: [1.0,1.0].into(),
            depth: -1.0,
            tex_indices: vec![vec![index]],
            next: None,
            flip: false,
        }
    }
    pub fn composite(start:usize, shape: V2<usize>, ss_width: usize) -> Self {
        let mut ti = Vec::new();
        for c in 0..shape.x {
            let column = (0..shape.y).map(|n|{n*ss_width + start + c}).collect();
            println!("{:?}",column);
            ti.push(column);
        }
        Self {
            pos: [0.0,0.0].into(),
            scale: [1.0,1.0].into(),
            depth: -1.0,
            tex_indices: ti,
            next: None,
            flip: false,
        }
    }
    pub fn empty() -> Self {
        Self {
            pos: [0.0,0.0].into(),
            scale: [0.0,0.0].into(),
            depth: -1.0,
            tex_indices: Vec::new(),
            next: None,
            flip: false
        }
    }
    pub fn repeat(mut self, times: usize) -> Self {
        let original = self.tex_indices.clone();
        for _ in 1..times {
            self.tex_indices.append(&mut original.clone())
        }
        self
    }
    pub fn with_pos(mut self, pos: V2<f32>) -> Self {
        self.pos = pos;
        self
    }
    pub fn with_scale(mut self, scale: V2<f32>) -> Self {
        self.scale = scale;
        self
    }
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }
    pub fn next(&mut self, sprite: Sprite) {
        if let Some(ref mut this_next) = self.next {
            this_next.next(sprite);
        } else {
            self.next = Some(Box::new(sprite));
        }
    }
}


pub struct SpriteSheet {
    pub tex: Option<glium::Texture2d>,
    pub shape: (usize,usize)
}

impl SpriteSheet {
    pub fn new(shape: (usize,usize)) -> Self {
        Self {
            tex: None,
            shape: shape
        }
    }
    pub fn vertices(&self, s: Sprite) -> Vec<Vertex> {

        let mut vertices = Vec::new();
        let mut current = &Some(Box::new(s.clone()));
        while let Some(sprite) = current {
            let tex_unit = (1.0 / self.shape.0 as f32, 1.0 / self.shape.1 as f32);
        

            for row in 0..sprite.tex_indices.len() {
                for col in 0..sprite.tex_indices[row].len() {
                    
                    let this_idx = sprite.tex_indices[if sprite.flip {sprite.tex_indices.len()-1-row}else{row}][col];

                    let mut tex_col: V2<f32> = [(this_idx%self.shape.0) as f32 * tex_unit.0, (this_idx%self.shape.0 + 1) as f32 * tex_unit.0].into();
                    let tex_row: V2<f32> = [(this_idx/self.shape.0) as f32 * tex_unit.1, (this_idx/self.shape.0 + 1) as f32 * tex_unit.1].into();

                    if sprite.flip {
                        let x = tex_col.x;
                        tex_col.x = tex_col.y;
                        tex_col.y = x;


                        /*let x = tex_row.x;
                        tex_row.x = tex_row.y;
                        tex_row.y = x;*/
                    }

                    vertices.append(&mut vec![
                        Vertex { pos: [sprite.pos.x+sprite.scale.x*row as f32, sprite.pos.y+sprite.scale.y*col as f32, sprite.depth], tex_coords: [tex_col.x, tex_row.x], rotation: 0.0 },
                        Vertex { pos: [sprite.pos.x+sprite.scale.x*(row+1) as f32, sprite.pos.y+sprite.scale.y*col as f32, sprite.depth], tex_coords: [tex_col.y, tex_row.x], rotation: 0.0 },
                        Vertex { pos: [sprite.pos.x+sprite.scale.x*(row+1) as f32, sprite.pos.y+sprite.scale.y*(col+1) as f32, sprite.depth], tex_coords: [tex_col.y, tex_row.y], rotation: 0.0 },

                        Vertex { pos: [sprite.pos.x+sprite.scale.x*row as f32, sprite.pos.y+sprite.scale.y*col as f32, sprite.depth], tex_coords: [tex_col.x, tex_row.x], rotation: 0.0 },
                        Vertex { pos: [sprite.pos.x+sprite.scale.x*row as f32, sprite.pos.y+sprite.scale.y*(col+1) as f32, sprite.depth], tex_coords: [tex_col.x, tex_row.y], rotation: 0.0 },
                        Vertex { pos: [sprite.pos.x+sprite.scale.x*(row+1) as f32, sprite.pos.y+sprite.scale.y*(col+1) as f32, sprite.depth], tex_coords: [tex_col.y, tex_row.y], rotation: 0.0 },
                    ]);
                }
            }

            current = &sprite.next;

        }


        vertices
    }
}



