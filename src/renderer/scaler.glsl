vec4 get_pixel(
    Sampler2D tex, 
    vec2 f_pos, 
    vec2 f_texel,
    float texel_size,
    float texel_size_2){

    vec2 scaled = floor(f_texel) * texel_size;
    
    vec2 quadrant = f_pos - (scaled + vec2(texel_size_2,texel_size_2));

    int off_x = quadrant.x > 0 ? 1 : 0;
    int off_y = quadrant.y > 0 ? 1 : 0;

    //distansen fra hjørnet
    vec2 dist = abs(f_pos - (scaled + vec2(float(off_x),float(off_y)) * texel_size));

    ivec2 v00 = ivec2(int(f_texel.x) + off_x,int(f_texel.y) + off_y);
    ivec2 v10 = ivec2(int(f_texel.x) + (off_x + 1) % 2,int(f_texel.y) + off_y);
    ivec2 v11 = ivec2(int(f_texel.x) + (off_x + 1) % 2,int(f_texel.y) + (off_y + 1) % 2);
    ivec2 v01 = ivec2(int(f_texel.x) + off_x,int(f_texel.y) + (off_y + 1) % 2);


    vec4 c00 = texelFetch(tex,v00,0);
    vec4 c10 = texelFetch(tex,v10,0); 
    vec4 c11 = texelFetch(tex,v11,0); 
    vec4 c01 = texelFetch(tex,v01,0); 

    if (c00.a > 0.0){{
        if (c10.a > 0.0 || c01.a > 0.0){{
            f_color = c00;
        }}
        else{{
            if (dist.x + dist.y < {0}){{
                f_color = c00;
            }}
            else{{
                f_color = vec4(0.0,0.0,0.0,0.0);
            }}
        }}
    }}
    else{{
        if (c01.a > 0.0 && c10.a > 0.0 && c11.a > 0.0 && dist.x + dist.y > texel_size_2){{
            f_color = (c01 + c10 + c11) / 3.0;
        }}
        else{{
            f_color = vec4(0.0,0.0,0.0,0.0);
        }}
    }}
}