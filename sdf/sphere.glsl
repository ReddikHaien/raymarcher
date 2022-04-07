float sdf_sphere(in vec3 position, in vec3 center, float radius){
    return distance(position,center) - radius;
}

bool bound_sphere(in vec3 origin, in vec3 ray, in vec3 center, float radius){
    vec3 L = center - origin;
    float vl = dot(L,ray);

    if (vl < 0.0) return false;  

    float d = dot(L,L) - vl*vl;
    return d <= radius*radius;
}


vec4 color_sphere(in vec3 position, in vec3 color){
    return vec4(color,1.0);
}