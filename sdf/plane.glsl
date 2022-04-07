float sdf_plane(in vec3 position, in vec3 normal, float height){
    return dot(position,normal) - height;
}

bool bound_plane(in vec3 origin, in vec3 ray, in vec3 normal,float height){
    float denom = dot(ray,normal);
    if (abs(denom) <= 0.0001){
        return false;
    }
    else{
        float dist = dot(vec3(0.0,height,0.0) - origin, normal) / denom;
        return dist >= 0.0001 && dist < 1000.0;
    }
}

vec4 color_plane(in vec3 position){
    return vec4(1.0,1.0,0.0,1.0);
}