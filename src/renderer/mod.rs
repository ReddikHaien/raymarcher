use std::{fs, mem::MaybeUninit, time::Instant};

use miniquad::{Context, EventHandler, PassAction};


use crate::renderer::scene::SceneSerializer;

use self::{methods::{MethodDefinition, DataDeserializer}, scene::{SceneInstance, Scene}, algorithms::RayMarcherBackend};

pub mod methods;
pub mod scene;
pub mod algorithms;

pub struct Renderer<S: Scene, R: RayMarcherBackend, A: App>{
    registered_bounding_methods: Vec<(String,DataDeserializer)>,
    registered_sdf_methods: Vec<(String,DataDeserializer)>,
    registered_tex_methods: Vec<(String,DataDeserializer)>,
    functionality: Vec<String>,
    timer: Instant,
    old: f32,
    frames: u32,
    scene: S,
    backend: R,
    app: MaybeUninit<A>
}

impl<S: Scene, R: RayMarcherBackend, A: App> Renderer<S, R, A> {
    pub fn new(ctx: &mut Context,scene: S, mut app: A) -> Self
    {
        let mut x = Self{
            functionality: Vec::new(),
            registered_bounding_methods: Vec::new(),
            registered_sdf_methods: Vec::new(),
            registered_tex_methods: Vec::new(),
            timer: Instant::now(),
            frames: 0,
            old: 0.0,
            scene,
            backend: R::new(ctx),
            app: MaybeUninit::uninit()
        };
        app.init(&mut x);
        x.app = MaybeUninit::new(app);
        let  fragment = x.get_scene_shader();
        println!("{}",fragment);
        x.backend.recreate_scene_shader(ctx, fragment);
        x
    }

    pub fn add_methods(&mut self, methods: MethodDefinition){
        let src = match methods{
            MethodDefinition::File(path) => fs::read_to_string(path).unwrap(),
            MethodDefinition::Script(src) => src,
        };
        self.functionality.push(src);
    }



    pub fn register_bound_method(&mut self, method_name: String, deserializer: DataDeserializer) -> u32{
        self.registered_bounding_methods.push((method_name,deserializer));
        self.registered_bounding_methods.len() as u32
    }

    pub fn register_sdf_method(&mut self, method_name: String, deserializer: DataDeserializer) -> u32{
        self.registered_sdf_methods.push((method_name,deserializer));
        self.registered_sdf_methods.len() as u32
    }

    pub fn register_tex_method(&mut self, method_name: String, deserializer: DataDeserializer) -> u32{
        self.registered_tex_methods.push((method_name,deserializer));
        self.registered_tex_methods.len() as u32
    }

    fn get_scene_shader(&self ) -> String{

        format!("#version 330

        in vec2 f_pos;
        
        out vec4 f_color;
        
        uniform float elapsed_time;

        uniform vec3 position;
        uniform vec4 rotation;

        uniform int scene_rom[1024];
        
        //method definitions
        {0}
        
        struct HitInfo{{
            float dist;
            int id;
        }};
        
        HitInfo sdf_scene(in vec3 origin, in vec3 position, in vec3 ray){{
            int pnt = 0;
            HitInfo curHit = HitInfo(1001.0,0);
        
            bool running = true;
            while(running){{
                int bound_type = scene_rom[pnt];
                pnt += 1;
                bool hitable = true;
                switch(bound_type){{
                    case 0: break;
                    {1}
                    default: break;
                }}

                
                int sdf_type = scene_rom[pnt];
                pnt += 1;
                switch (sdf_type){{
                    case 0: {{
                        running = false;
                    }}break;

                    {2}

                    default: {{
                        running = false;
                    }}break;
                }}
                
            }}
        
            return curHit;
        }}
        
        vec4 color(int pnt, in vec3 position){{
            int tex_type = scene_rom[pnt];
            pnt += 1;
            switch (tex_type){{
                case 0: return vec4(1.0,0.0,1.0,1.0);
                {3}
                default: return vec4(1.0,0.0,1.0,1.0);
            }}
        }}
        
        void main(){{
        
            vec3 ray = normalize(vec3(f_pos,2.0));
            
            vec3 hit_position = position;
            HitInfo cur = HitInfo(0.0,0);
            float traveled = 0.0;
            float u = 255.0;
            for (int i = 0; i < 256; i++){{
                cur = sdf_scene(position, hit_position, ray);
                traveled += cur.dist;
                hit_position = position + ray * traveled;
                if (cur.dist < 0.01 || traveled > 1000.0){{
                    u = float(i);
                    break;
                }}
            }}
        
            if (cur.dist < 0.01){{
                f_color = color(cur.id,hit_position);
        
            }}
            else{{
                u /= 64;
                f_color = vec4(u,u,u,1.0);
            }}
        }}
        ",
        self.functionality.join("\n"),
        self.registered_bounding_methods.iter().enumerate().map(|(id,(name,deserializer))| deserializer.create_bounding_case(id as u32 + 1, name)).collect::<Vec<String>>().join("\n"),
        self.registered_sdf_methods.iter().enumerate().map(|(id,(name,deserializer))| deserializer.create_sdf_case(id as u32 + 1, name)).collect::<Vec<String>>().join("\n"),
        self.registered_tex_methods.iter().enumerate().map(|(id,(name,deserializer))| deserializer.create_tex_case(id as u32 + 1, name)).collect::<Vec<String>>().join("\n")
        )
    }

    
}


impl<T: Scene, R: RayMarcherBackend, A: App> EventHandler for Renderer<T, R, A>{
    fn update(&mut self, _ctx: &mut miniquad::Context) {
        unsafe{
            let app = self.app.assume_init_mut();
            app.update(&mut self.scene, &mut self.backend);
        }
    }

    fn draw(&mut self, ctx: &mut miniquad::Context) {
        if self.scene.dirty(){
            let rom = self.backend.get_scene_rom();
            let mut serializer = SceneSerializer::new(rom);
            self.scene.serialize(&mut serializer);
            self.scene.mark_clean();
        }
        let elapsed = self.timer.elapsed().as_secs_f32();
        self.backend.set_elapsed(elapsed);
        if elapsed - self.old > 1.0{
            println!("{}: {}",(elapsed - self.old),self.frames);
            self.frames = 0;
            self.old = elapsed;
        }
        self.frames += 1;
        self.backend.render(ctx);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.backend.resize(ctx, width, height);  
    }
    fn key_down_event(&mut self, ctx: &mut Context, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods, repeat: bool) {
        unsafe{ self.app.assume_init_mut().key_down_event(ctx, keycode, keymods, repeat); }
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods) {
        unsafe{ self.app.assume_init_mut().key_up_event(ctx, keycode, keymods); }
    }
}

pub trait App {
    fn init<S: Scene, B: RayMarcherBackend>(&mut self, renderer: &mut Renderer<S, B, Self>)
        where Self: Sized;
    fn update<S: Scene, B: RayMarcherBackend>(&mut self,scene: &mut S, backend: &mut B);

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods, _repeat: bool) {
        
    }
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods) {
        
    }
}