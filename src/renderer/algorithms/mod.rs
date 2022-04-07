use miniquad::Context;

pub use scaled_estimate_backend::*;
pub use full_size_backend::*;

mod scaled_estimate_backend;
mod full_size_backend;
pub trait RayMarcherBackend{
    fn new(ctx: &mut Context) -> Self;
    fn resize(&mut self, ctx: &mut miniquad::Context, width: f32, height: f32);
    fn render(&mut self, ctx: &mut Context);
    fn set_elapsed(&mut self, time: f32);
    fn set_position(&mut self, position: [f32;3]);
    fn set_rotation(&mut self, rotation: [f32;4]);
    fn get_scene_rom(&mut self) -> &mut [u32];
    fn recreate_scene_shader(&mut self, ctx: &mut Context, fragment: String);
}

#[repr(C)]
struct SceneUniformShader{
    pub fov_y: f32,
    pub elapsed_time: f32,
    pub position: [f32;3],
    pub rotation: [f32;4],
    pub scene_rom: [u32;1024]
}

impl SceneUniformShader{
    pub fn new() -> Self{
        Self{
            elapsed_time: 0.0,
            fov_y: 1.0,
            position: [0.0,0.0,0.0],
            rotation: [0.0,0.0,0.0,1.0],
            scene_rom: [0;1024]
        }
    }
}

const VERTS: [f32;8] = [
    -1.0,-1.0,
    1.0,-1.0,
    -1.0,1.0,
    1.0,1.0
];

const INDICES: [u16;6] = [
    0,1,2,
    2,1,3
];