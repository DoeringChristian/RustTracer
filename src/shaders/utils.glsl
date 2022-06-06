#ifndef UTIL_GLSL
#define UTIL_GLSL
#include "types.glsl"

const float PI = 3.14159265358979323846264338327950288;

// Generate random value in range 0..1
float rand(float seed){
    highp float c = 43758.5453;
    highp float dt= seed;
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}
float rand(vec2 seed){
    highp float a = 12.9898;
    highp float b = 78.233;
    highp float c = 43758.5453;
    highp float dt= dot(seed.xy ,vec2(a,b));
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

bool intersects_aabb(Ray ray, vec4 bmin, vec4 bmax){
    vec3 tmin = (bmin.xyz - ray.pos.xyz) / ray.dir.xyz;
    vec3 tmax = (bmax.xyz - ray.pos.xyz) / ray.dir.xyz;
    vec3 t1 = min(tmin, tmax);
    vec3 t2 = max(tmin, tmax);
    float tnear = max(max(t1.x, t1.y), t1.z);
    float tfar = min(min(t2.x, t2.y), t2.z);

    if(tnear > tfar || tfar <= 0)
        return false;
    return true;
}

float mix2(float v0, float v1, float v2, float u, float v){
    return (1. - u - v)  * v0 + v1 * u + v2 * v;
}
vec2 mix2(vec2 v0, vec2 v1, vec2 v2, float u, float v){
    return (1. - u - v)  * v0 + v1 * u + v2 * v;
}
vec3 mix2(vec3 v0, vec3 v1, vec3 v2, float u, float v){
    return (1. - u - v)  * v0 + v1 * u + v2 * v;
}
vec4 mix2(vec4 v0, vec4 v1, vec4 v2, float u, float v){
    return (1. - u - v)  * v0 + v1 * u + v2 * v;
}

vec3 triangle_uvt(Ray ray, vec3 v0, vec3 v1, vec3 v2){
    vec3 v01 = v1 - v0;
    vec3 v02 = v2 - v0;
    mat3 M = inverse(mat3(v01, v02, -ray.dir.xyz));
    return M * (ray.pos.xyz - v0);
}
#endif
