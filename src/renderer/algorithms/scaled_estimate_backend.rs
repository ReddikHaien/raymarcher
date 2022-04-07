use miniquad::{Pipeline, Bindings, RenderPass, Context, BufferType, Buffer, Shader, UniformBlockLayout, UniformDesc, BufferLayout, VertexAttribute, VertexFormat, Texture, TextureParams, FilterMode, UniformType, ShaderMeta, PassAction};

use super::{SceneUniformShader, RayMarcherBackend, VERTS, INDICES};

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

uniform int scene_rom[1024];

float sdf_sphere(in vec3 p, in vec3 pos, float radius){
    return length(p - pos) - radius;
}

float sdf_plane(in vec3 p, in vec3 normal, float height){
    return dot(p,normal) - height;
}

struct HitInfo{
    float dist;
    int id;
};

HitInfo sdf_scene(in vec3 p){
    int pnt = 0;
    HitInfo curHit = HitInfo(1000.0,0);

    bool running = true;
    while(running){
        int type = scene_rom[pnt];
        pnt += 1;
        switch(type){
            case 0: running = false; break; //End of Array

            case 1: {
                vec3 pos = vec3(
                    intBitsToFloat(scene_rom[pnt]),
                    intBitsToFloat(scene_rom[pnt+1]),
                    intBitsToFloat(scene_rom[pnt+2])
                );
                float radius = intBitsToFloat(scene_rom[pnt+3]);
                pnt += 4;
                float new = sdf_sphere(p, pos, radius);
                if (new < curHit.dist){ 
                    curHit = HitInfo(new,type);
                }
            } break;

            case 2: {
                vec3 normal = vec3(
                    intBitsToFloat(scene_rom[pnt]),
                    intBitsToFloat(scene_rom[pnt+1]),
                    intBitsToFloat(scene_rom[pnt+2])
                );
                float height = intBitsToFloat(scene_rom[pnt+3]);
                pnt += 4;
                float new = sdf_plane(p, normal, height);
                if (new < curHit.dist){ 
                    curHit = HitInfo(new,type);
                }
            } break;
        }
    }

    return curHit;
}


vec4 sphere_color(in vec3 position){
    float c = floor(position.x) + floor(position.y) + floor(position.z);
    c = fract(c * 0.5);
    c *= 2;
    return vec4(vec3(0.4,0.4,0.0) * c + vec3(0.6),1.0);
}

vec4 color(int id, in vec3 position){
    vec4 output_color = vec4(0.0);
    switch (id) {
        case 1:
            output_color = sphere_color(position);
            break;
        case 2:
            output_color = vec4(0.6,0.1,0.1,1.0);
            break;
    }

    return output_color;
}

void main(){

    vec3 ray = normalize(vec3(f_pos,2.0));
    vec3 origin = vec3(0.0,0.0,0.0);
    HitInfo cur = HitInfo(0.0,0);
    float traveled = 0.0;
    for (int i = 0; i < 256; i++){
        cur = sdf_scene(origin);
        traveled += cur.dist;
        origin = origin + ray * cur.dist;
        if (cur.dist < 0.01 || traveled > 1000.0){
            break;
        }
    }

    if (cur.dist < 0.01){
        f_color = color(cur.id,origin);

    }
    else{
        f_color = vec4(0.0,0.0,0.0,1.0);
    }
}
";


pub const SCREEN_SCALING: u32 = 2;

pub struct ScaledEstimateBackend{
    scene_pipeline: Pipeline,
    scene_bind: Bindings,
    scene_pass: RenderPass,

    display_pipeline: Pipeline,
    display_bind: Bindings,

    uniforms: SceneUniformShader,
}

impl RayMarcherBackend for ScaledEstimateBackend{
    
    fn new(ctx: &mut miniquad::Context) -> Self {
        let render_width = 800 / SCREEN_SCALING;
        let render_height = 600 / SCREEN_SCALING;

        let (color, depth) = Self::get_render_textures(ctx, render_width, render_height);

        let scene_pass = RenderPass::new(ctx, color, depth);

        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &VERTS);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &INDICES);

        let scene_bind = Bindings{
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![]
        };

        let scene_shader = Shader::new(ctx, VERTEX_SHADER, &FRAGMENT_SHADER,ShaderMeta{
            images: vec![],
            uniforms: UniformBlockLayout{
                uniforms: vec![
                    UniformDesc::new("fov_y",UniformType::Float1),
                    UniformDesc::new("elapsed_time", UniformType::Float1),
                    UniformDesc::new("position", UniformType::Float3),
                    UniformDesc::new("rotation", UniformType::Float4),
                    UniformDesc::new("scene_rom", UniformType::Int1).array(1024)
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


        //Window renderer
        let display_bind = Bindings{
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![color]
        };

        let display_pipeline = Self::get_display_pipeline(ctx, 800.0, 600.0);

        Self{
            scene_pipeline,
            scene_bind,
            scene_pass,

            display_bind,
            display_pipeline,
            uniforms: SceneUniformShader{
                fov_y: 600.0/800.0,
                elapsed_time: 0.0,
                position: [0.0,0.0,0.0],
                rotation: [0.0,0.0,0.0,1.0],
                scene_rom: [0;1024]
            },
        }
    }

    fn resize(&mut self, ctx: &mut miniquad::Context, width: f32, height: f32) {
        self.uniforms.fov_y = height / width;
        self.recreate_scene_targets(ctx, width, height)   
    }

    fn render(&mut self, ctx: &mut miniquad::Context) {
        ctx.begin_pass(self.scene_pass, PassAction::clear_color(0.0, 0.0, 0.0, 0.0));
        ctx.apply_pipeline(&self.scene_pipeline);
        ctx.apply_bindings(&self.scene_bind);
        ctx.apply_uniforms(&self.uniforms);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();

        ctx.begin_default_pass(PassAction::clear_color(1.0, 1.0, 1.0, 1.0));
        ctx.apply_pipeline(&self.display_pipeline);
        ctx.apply_bindings(&self.display_bind);
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

    fn recreate_scene_shader(&mut self, ctx: &mut Context, fragment: String){
        let scene_shader = Shader::new(ctx, VERTEX_SHADER, &fragment,ShaderMeta{
            images: vec![],
            uniforms: UniformBlockLayout{
                uniforms: vec![
                    UniformDesc::new("fov_y",UniformType::Float1),
                    UniformDesc::new("elapsed_time", UniformType::Float1),
                    UniformDesc::new("position", UniformType::Float3),
                    UniformDesc::new("rotation", UniformType::Float4),
                    UniformDesc::new("scene_rom", UniformType::Int1).array(1024)
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

impl ScaledEstimateBackend{
    fn recreate_scene_targets(&mut self, ctx: &mut Context, width: f32, height: f32){

        let scaled_width = (width as u32) / SCREEN_SCALING;
        let scaled_height = (height as u32) / SCREEN_SCALING;

        //Offscren renderer
        let (color, depth) = Self::get_render_textures(ctx, scaled_width, scaled_height);

        let scene_pass = RenderPass::new(ctx, color, depth);

        let display_bind = Bindings{
            vertex_buffers: vec![self.display_bind.vertex_buffers[0].clone()],
            index_buffer: self.display_bind.index_buffer.clone(),
            images: vec![color]
        };

        let display_pipeline = Self::get_display_pipeline(ctx, width, height);

        self.scene_pass = scene_pass;
        self.display_bind = display_bind;
        self.display_pipeline = display_pipeline;
    }

    fn get_render_textures(ctx: &mut Context, width: u32, height: u32) -> (Texture,Texture){
        let color = Texture::new_render_texture(ctx, TextureParams{
            width,
            height,
            format: miniquad::TextureFormat::RGBA8,
            ..Default::default()
        });

        let depth = Texture::new_render_texture(ctx,TextureParams{
            width,
            height,
            format: miniquad::TextureFormat::Depth,
            ..Default::default()
        });

        color.set_filter(ctx, FilterMode::Nearest);
        depth.set_filter(ctx, FilterMode::Nearest);
        
        (color,depth)
    }

    fn get_display_pipeline(ctx: &mut Context, width: f32, height: f32) -> Pipeline{

        let vertex = Self::get_display_vertex_shader(width, height);
        let fragment = Self::get_display_fragment(width, height);
        let display_shader = Shader::new(ctx, &vertex, &fragment, ShaderMeta{
            uniforms: UniformBlockLayout{
                uniforms: Vec::new()
            },
            images: vec!["tex".to_string()]
        }).unwrap_or_else(|e| panic!("Failed to compile display shader: {}",e.to_string()));


        let display_pipeline = Pipeline::new(
            ctx, 
            &[BufferLayout::default()], 
            &[
                VertexAttribute::new("pos", VertexFormat::Float2)
            ],
            display_shader);

        display_pipeline
    }


    fn get_display_vertex_shader(width: f32, height: f32) -> String{
        format!("#version 330
        in vec2 pos;
        out vec2 f_pos;
        out vec2 f_texel;
        
        void main(){{
            
            gl_Position = vec4(pos,0.1,1.0);
            f_texel = vec2(floor((pos.x * {0} + {0}) / {2}),       floor((pos.y * {1} + {1}) / {2}) );
            f_pos = vec2(  floor((pos.x * {0} + {0}) / {2}) * {2}, floor((pos.y * {1} + {1}) / {2}) * {2});
            //f_pos = (pos * 0.5) + vec2(0.5,0.5);
        }}
        ",width / 2.0, height / 2.0, SCREEN_SCALING)
    }

    fn get_display_fragment(width: f32, height: f32) -> String{
        format!(
            "#version 330
in vec2 f_pos;
in vec2 f_texel;

uniform sampler2D tex;

out vec4 f_color;

void main(){{

    vec2 top_left = floor(f_texel) * {0};
    vec2 top_right = top_left + vec2({0},0.0);
    vec2 bot_left = top_left + vec2({0},{0});
    vec2 bot_right = top_left + vec2({0},{0});

    vec2 quadrant = f_pos - (top_left + vec2({1},{1}));

    ivec2 pixel = ivec2(int(f_texel.x)-1,int(f_texel.y)-1);

    vec4 cbl = texelFetch(tex,pixel + ivec2(0,0),0);
    vec4 cbm = texelFetch(tex,pixel + ivec2(1,0),0);
    vec4 cbr = texelFetch(tex,pixel + ivec2(2,0),0);
    
    vec4 cml = texelFetch(tex,pixel + ivec2(0,1),0);
    vec4 cmm = texelFetch(tex,pixel + ivec2(1,1),0);
    vec4 cmr = texelFetch(tex,pixel + ivec2(2,1),0);

    vec4 ctl = texelFetch(tex,pixel + ivec2(0,2),0);
    vec4 ctm = texelFetch(tex,pixel + ivec2(1,2),0);
    vec4 ctr = texelFetch(tex,pixel + ivec2(2,2),0);
    
    //l m r
    //##XX## t
    //##XX##
    //XX01XX m
    //XX23XX
    //##XX## b
    //##XX##

    if (cmm.a == 0.0){{
        f_color = vec4(0.0,0.0,0.0,0.0);
    }}
    else{{
        if (quadrant.x < 0){{
            if (quadrant.y < 0){{

                vec2 dist = abs(f_pos - bot_left);
                if (cbm != cmm && cml != cmm && ctm == cmm && cmr == cmm){{
                    
                    if ((dist.x + dist.y) / {0}  <  1.414){{
                        f_color = cmm;
                    }}
                    else{{
                        f_color = (cbm + cml) / 2.0;
                    }}
                }}
                else{{
                    f_color = cmm;
                }}

            }}
            else{{
                vec2 dist = abs(f_pos - top_left);
                dist.x = {1} - dist.x;
                if (ctm != cmm && cml != cmm && cbm == cmm && cmr == cmm){{
                    
                    if (dist.x + dist.y < {0}){{
                        f_color = cmm;
                    }}
                    else{{
                        f_color = (ctm + cml) / 2.0;
                    }}
                }}
                else{{
                    f_color = cmm;
                }}
            }}
        }}
        else{{
            if (quadrant.y < 0){{
                
                vec2 dist = abs(f_pos - bot_right);
                if (cbm != cmm && cmr != cmm && ctm == cmm && cml == cmm){{

                    dist.y = {0} - dist.y;

                    if (abs(dist.y + dist.x) > {1}){{
                        f_color = cmm;
                    }}
                    else{{
                        f_color = (cbm + cmr) / 2.0;
                    }}
                }}
                else{{
                    f_color = cmm;
                }}

            }}
            else{{
                vec2 dist = abs(f_pos - top_right);
                dist.x = {1} - dist.x;
                if (ctm != cmm && cmr != cmm && cbm == cmm && cml == cmm){{
                    
                    if (dist.x + dist.y < {0}){{
                        f_color = cmm;
                    }}
                    else{{
                        f_color = (ctm + cmr) / 2.0;
                    }}
                }}
                else{{
                    f_color = cmm;
                }}
            }}
        }}
    }}

    f_color.a = 1.0;


    //f_color = textureBicubic(tex,f_pos);

}}

",SCREEN_SCALING, SCREEN_SCALING as f32 / 2.0)
    }
}