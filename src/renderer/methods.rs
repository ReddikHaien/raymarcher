use std::{path::PathBuf, fmt::format};

use miniquad::UniformType;

pub enum MethodDefinition{
    File(PathBuf),
    Script(String),   
}

pub struct DataEntry{
    pub name: String,
    pub type_: UniformType
}

impl ToString for DataEntry{
    fn to_string(&self) -> String {
        let (init,len,type_) = match self.type_{
            UniformType::Float1 => ("intBitsToFloat(scene_rom[pnt])",1,"float"),
            UniformType::Float2 => ("vec2(intBitsToFloat(scene_rom[pnt]),intBitsToFloat(scene_rom[pnt+1]))",2,"vec2"),
            UniformType::Float3 => ("vec3(
                intBitsToFloat(scene_rom[pnt]),
                intBitsToFloat(scene_rom[pnt+1]),
                intBitsToFloat(scene_rom[pnt+2]))",3,"vec3"),
            UniformType::Float4 => ("vec4(
                intBitsToFloat(scene_rom[pnt]),
                intBitsToFloat(scene_rom[pnt+1]),
                intBitsToFloat(scene_rom[pnt+2]),
                intBitsToFloat(scene_rom[pnt+3]))",4,"vec4"),
            UniformType::Int1 => ("scene_rom[pnt]",1,"int"),
            UniformType::Int2 => ("ivec2(
                scene_rom[pnt],
                scene_rom[pnt+1]
            )",2,"ivec2"),
            UniformType::Int3 => ("ivec3(
                scene_rom[pnt],
                scene_rom[pnt+1],
                scene_rom[pnt+2]
            )",3,"ivec3"),
            UniformType::Int4 => ("ivec4(
                scene_rom[pnt],
                scene_rom[pnt+1],
                scene_rom[pnt+2],
                scene_rom[pnt+3]
            )",4,"ivec4"),
            UniformType::Mat4 => unimplemented!(),
        };

        format!("{} {} = {}; pnt += {};",type_,self.name,init,len)
    }
}

pub struct DataDeserializer{
    pub entries: Vec<DataEntry>
}
impl DataDeserializer{
    pub fn create_bounding_case(&self, id: u32, name: &str) -> String{
        let values = self.entries.iter().map(|x|x.to_string()).collect::<Vec<_>>().join("\n");
        let value_names = ["origin".to_string(), "ray".to_string()].into_iter().chain(self.entries.iter().map(|x|x.name.clone())).collect::<Vec<String>>().join(", ");
        format!(
            "
            case {}: {{
                {}

                hitable = {}({});
            }} break;
            ",
            id,
            values,
            name,
            value_names,
        )
    }

    pub fn create_sdf_case(&self, id: u32, name: &str) -> String{
        let values = self.entries.iter().map(|x|x.to_string()).collect::<Vec<_>>().join("\n");
        let value_names = ["position".to_string()].into_iter().chain(self.entries.iter().map(|x|x.name.clone())).collect::<Vec<String>>().join(", ");
        format!(
            "
            case {}: {{
                {}
                
                int tex_pnt = pnt+1;
                pnt += 1 + scene_rom[pnt];

                if (hitable){{
                    float new = {}({});
                    if (new < curHit.dist){{ 
                        curHit = HitInfo(new,tex_pnt);
                    }}
                }}
            }} break;
            ",
            id,
            values,
            name,
            value_names,
        )
    }

    pub fn create_tex_case(&self, id: u32, name: &str) -> String{
        let values = self.entries.iter().map(|x|x.to_string()).collect::<Vec<_>>().join("\n");
        let value_names = ["position".to_string()].into_iter().chain(self.entries.iter().map(|x|x.name.clone())).collect::<Vec<String>>().join(", ");
        
        format!(
            "case {}:{{
                {}
                return {}({}); 
            }} break;",
            id,
            values,
            name,
            value_names
        )
    }
}