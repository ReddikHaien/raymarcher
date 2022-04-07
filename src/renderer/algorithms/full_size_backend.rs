use miniquad::{Pipeline, BufferLayout, VertexAttribute, UniformDesc, UniformType, UniformBlockLayout, Shader, Bindings, Buffer, BufferType, ShaderMeta, VertexFormat, PassAction};

use crate::renderer::MAX_ROM_SIZE;

use super::{RayMarcherBackend, VERTS, INDICES, SceneUniformShader};

const VERTEX_SHADER: &'static str = 
"#version 330
in vec2 pos;

out vec2 f_pos;

uniform float fov_y;

void main(){
    gl_Position = vec4(pos,0.1,1.0);
    f_pos = vec2(pos.x,pos.y * fov_y);
}";

const FRAGMENT_SHADER: &'static str = 
"#version 330

in vec2 f_pos;

out vec4 f_color;

uniform int scene_rom[3072];

uniform float elapsed_time;

void main(){
    f_color = vec4(0,0,0,1);
}
";


pub struct FullSizeBackend{
    scene_pipeline: Pipeline,
    scene_bind: Bindings,
    uniforms: SceneUniformShader,
}

impl RayMarcherBackend for FullSizeBackend {
    
    fn new(ctx: &mut miniquad::Context) -> Self {
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &VERTS);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &INDICES);

        let scene_bind = Bindings{
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![]
        };

        let (w,h) = ctx.screen_size();
        let fov_y = h / w;

        let scene_shader = Shader::new(ctx, VERTEX_SHADER, &FRAGMENT_SHADER,ShaderMeta{
            images: vec![],
            uniforms: UniformBlockLayout{
                uniforms: vec![
                    UniformDesc::new("fov_y",UniformType::Float1),
                    UniformDesc::new("elapsed_time", UniformType::Float1),
                    UniformDesc::new("position", UniformType::Float3),
                    UniformDesc::new("rotation", UniformType::Float4),
                    UniformDesc::new("scene_rom", UniformType::Int1).array(MAX_ROM_SIZE)
                ],
            }
        }).unwrap_or_else(|e| panic!("Failed to compile scene shader: {}",e));

        let scene_pipeline = Pipeline::new(
            ctx, 
            &[BufferLayout::default()], 
            &[
                VertexAttribute::new("pos", VertexFormat::Float2)
                
            ],
        scene_shader);

        let mut uniforms = SceneUniformShader::new();

        uniforms.fov_y = fov_y;
        Self{
            scene_pipeline,
            scene_bind,
            uniforms
        }
    }

    fn resize(&mut self, ctx: &mut miniquad::Context, width: f32, height: f32) {
        self.uniforms.fov_y = height / width;
    }

    fn render(&mut self, ctx: &mut miniquad::Context) {
        ctx.begin_default_pass(PassAction::clear_color(1.0, 1.0, 1.0, 1.0));
        ctx.apply_pipeline(&self.scene_pipeline);
        ctx.apply_bindings(&self.scene_bind);
        ctx.apply_uniforms(&self.uniforms);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn set_elapsed(&mut self, time: f32) {
        self.uniforms.elapsed_time = time;
    }

    fn get_scene_rom(&mut self) -> &mut [u32] {
        &mut self.uniforms.scene_rom[..]
    }

    fn recreate_scene_shader(&mut self, ctx: &mut miniquad::Context, fragment: String) {
        let scene_shader = Shader::new(ctx, VERTEX_SHADER, &fragment,ShaderMeta{
            images: vec![],
            uniforms: UniformBlockLayout{
                uniforms: vec![
                    UniformDesc::new("fov_y",UniformType::Float1),
                    UniformDesc::new("elapsed_time", UniformType::Float1),
                    UniformDesc::new("position", UniformType::Float3),
                    UniformDesc::new("rotation", UniformType::Float4),
                    UniformDesc::new("scene_rom", UniformType::Int1).array(MAX_ROM_SIZE)
                ],
            }
        }).unwrap_or_else(|e| panic!("Failed to compile scene shader: {}",e));

        let scene_pipeline = Pipeline::new(
            ctx, 
            &[BufferLayout::default()], 
            &[
                VertexAttribute::new("pos", VertexFormat::Float2)
            ],
        scene_shader);

        self.scene_pipeline = scene_pipeline;
    }

    fn set_position(&mut self, position: [f32;3]) {
        self.uniforms.position = position;
    }

    fn set_rotation(&mut self, rotation: [f32;4]) {
        self.uniforms.rotation = rotation;
    }
}