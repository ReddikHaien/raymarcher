use std::{path::PathBuf, str::FromStr, collections::{HashMap, HashSet}};

use miniquad::{conf::Conf, EventHandler, Context, UserData, Pipeline, RenderPass, Texture, TextureParams, Buffer, BufferType, Bindings, Shader, ShaderMeta, UniformBlockLayout, BufferLayout, VertexAttribute, VertexFormat, PassAction, FilterMode, KeyMods, KeyCode};
use miniquad_raytrace::renderer::{Renderer, methods::{MethodDefinition, DataDeserializer, DataEntry}, scene::{SimpleScene, SceneInstance, Serializeable, Scene}, algorithms::{ScaledEstimateBackend, FullSizeBackend, RayMarcherBackend}, App};

struct SimpleSphere{
    pos: [f32;3],
    radius: f32
}

impl SimpleSphere{
    pub fn new(pos: [f32;3],radius: f32) -> Self{
        Self{
            pos,
            radius
        }
    }
}

impl Serializeable for SimpleSphere{
    fn serialize<'a>(&self, serializer: &mut miniquad_raytrace::renderer::scene::SceneSerializer<'a>) {
        serializer.write_value(1); // Bound Id
        self.pos.serialize(serializer); // Bound Data
        self.radius.serialize(serializer);
        serializer.write_value(1); // Sdf Id
        self.pos.serialize(serializer); // Sdf Data
        self.radius.serialize(serializer);
        serializer.write_value(4); // Tex Len
        serializer.write_value(1); // Tex Id
        [0.0f32,1.0,1.0].serialize(serializer);
    }
}

struct SimplePlane{
    normal: [f32;3],
    height: f32
}
impl SimplePlane{
    pub fn new(normal: [f32;3], height: f32) -> Self{
        Self{
            normal,
            height
        }
    }
}

impl Serializeable for SimplePlane{
    fn serialize<'a>(&self, serializer: &mut miniquad_raytrace::renderer::scene::SceneSerializer<'a>) {
        serializer.write_value(2); // Bound Id
        self.normal.serialize(serializer); // Bound Data
        self.height.serialize(serializer);
        serializer.write_value(2); // Sdf Id
        self.normal.serialize(serializer); // Sdf Data
        self.height.serialize(serializer);
        serializer.write_value(1); // Tex Len
        serializer.write_value(2); // Tex Id
    }
}

struct Logic{
    key_map: HashSet<KeyCode>,
    position: [f32;3],
    rotation: [f32;4],
}

impl<B: RayMarcherBackend> App<SimpleScene, B> for Logic{
    fn init(&mut self, renderer: &mut Renderer<SimpleScene, B, Self>)
        where Self: Sized {
            renderer.add_methods(MethodDefinition::File(PathBuf::from_str("./sdf/plane.glsl").unwrap()));
            renderer.add_methods(MethodDefinition::File(PathBuf::from_str("./sdf/sphere.glsl").unwrap()));

            let bound_sphere_id = renderer.register_bound_method("bound_sphere".to_string(), DataDeserializer{
                entries: vec![
                    DataEntry{ name: "center".into(), type_: miniquad::UniformType::Float3 },
                    DataEntry{ name: "radius".into(), type_: miniquad::UniformType::Float1 }
                ]
            });

            let bound_plane_id = renderer.register_bound_method("bound_plane".to_string(), DataDeserializer{
                entries: vec![
                    DataEntry{ name: "normal".into(), type_: miniquad::UniformType::Float3 },
                    DataEntry{ name: "height".into(), type_: miniquad::UniformType::Float1 }
                ]
            });

            let sphere_id = renderer.register_sdf_method("sdf_sphere".into(), DataDeserializer{
                entries: vec![
                    DataEntry{ name: "center".into(), type_: miniquad::UniformType::Float3 },
                    DataEntry{ name: "radius".into(), type_: miniquad::UniformType::Float1 }
                ]
            });

            let plane_id = renderer.register_sdf_method("sdf_plane".into(), DataDeserializer{
                entries: vec![
                    DataEntry{ name: "normal".into(), type_: miniquad::UniformType::Float3 },
                    DataEntry{ name: "height".into(), type_: miniquad::UniformType::Float1 }
                ]
            });

            let tex_sphere_id = renderer.register_tex_method("color_sphere".to_string(), DataDeserializer { 
                entries: vec![
                    DataEntry{ name: "sph_color".into(), type_: miniquad::UniformType::Float3 }
                ]
            });

            let tex_plane_id = renderer.register_tex_method("color_plane".to_string(), DataDeserializer { 
                entries: vec![]
            });
    }

    fn update(&mut self,scene: &mut SimpleScene, backend: &mut B) {
        if self.key_map.contains(&KeyCode::W){
            self.position[2] += 0.01;
        }
        if self.key_map.contains(&KeyCode::S){
            self.position[2] -= 0.01;
        }
        backend.set_position(self.position.clone());
    }
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods, _repeat: bool) {
        self.key_map.insert(keycode);
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods) {
        self.key_map.remove(&keycode);   
    }
}

fn main() {
    miniquad::start(
        Conf{
            window_title: "Test".into(),
            window_width: 800,
            window_height: 600,
            sample_count: 1,
            ..Default::default()
        },
        |mut ctx| {

            let mut scene = SimpleScene::new();

            scene.add_instance(SimpleSphere::new([-2.0,0.0,7.0], 1.0));
            scene.add_instance(SimpleSphere::new([2.0,0.0,7.0], 1.0));

            scene.add_instance(SimplePlane::new([0.0,1.0,0.0], -20.0));

            scene.mark_dirty();

            UserData::owning(Renderer::<_,FullSizeBackend,_>::new(&mut ctx, scene, Logic{
                position: [0.0;3],
                rotation: [0.0,0.0,0.0,1.0],
                key_map: HashSet::new(),
            }),ctx)
        }
    )
}
