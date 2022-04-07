use std::{ptr::NonNull, num::NonZeroU32};

pub trait Scene : Serializeable{
    fn dirty(&self) -> bool;
    fn mark_clean(&mut self);
}

pub trait SceneInstance{
    fn get_bound_id(&self) -> Option<NonZeroU32>;
    fn get_sdf_id(&self) -> u32;
    fn get_tex_id(&self) -> Option<NonZeroU32>;
    fn get_bound_data(&self) -> Vec<u32>;
    fn get_sdf_data(&self) -> Vec<u32>;
    fn get_tex_data(&self) -> Vec<u32>;
}

impl Serializeable for dyn SceneInstance{
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>) {
        match self.get_bound_id(){
            Some(x) => {
                serializer.write_value(x.get());
                self.get_bound_data()[..].serialize(serializer);
            },
            None => serializer.write_value(0),
        }
        serializer.write_value(self.get_sdf_id());
        self.get_sdf_data().serialize(serializer);
        match self.get_tex_id(){
            Some(x) => {
                let data = self.get_tex_data();
                serializer.write_value(1 + data.len() as u32);
                serializer.write_value(x.get());
                data.serialize(serializer);
            },
            None => {
                [1u32,0].serialize(serializer);
            },
        }
    }
}

pub trait Serializeable{
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>);
}

impl Serializeable for f32{
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>){
        serializer.write_value(self.to_bits());
    }
}

impl Serializeable for u32{
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>) {
        serializer.write_value(*self);
    }
}

impl Serializeable for Box<dyn Serializeable> {
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>) {
        self.as_ref().serialize(serializer);
    }
}

impl<T: Serializeable> Serializeable for [T] {
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>) {
        for x in self.iter(){
            x.serialize(serializer);
        }
    }
}

pub struct SceneSerializer<'a>{
    out: &'a mut[u32],
    index: usize
}

impl<'a> SceneSerializer<'a> {
    pub fn new(out: &'a mut [u32]) -> Self{
        Self{
            index: 0,
            out
        }
    }

    pub fn has_space_for(&mut self, els: usize) -> bool{
        self.index + els <= 1024
    }

    pub fn write_value(&mut self, value: u32){
        if self.has_space_for(1){
            self.out[self.index] = value;
            self.index+=1;    
        }
    }

    pub fn write_values(&mut self, value: &[u32]){
        if self.has_space_for(value.len()){
            self.out[self.index..].copy_from_slice(value);
        }
    }
}

pub struct SimpleScene{
    dirty: bool,
    objects: Vec<Box<dyn Serializeable>>,
}

impl SimpleScene{
    pub fn new() -> Self{
        Self{
            dirty: false,
            objects: vec![],
        }
    }

    pub fn add_instance(&mut self, x: impl Serializeable + 'static){
        self.objects.push(Box::new(x));
    }

    pub fn mark_dirty(&mut self){
        self.dirty = true;
    }
}

impl Serializeable for SimpleScene{
    fn serialize<'a>(&self, serializer: &mut SceneSerializer<'a>) {
        self.objects.iter().for_each(|x|{
            x.serialize(serializer);
        });
    }
}

impl Scene for SimpleScene{
    fn dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self){
        self.dirty = false;
    }
}