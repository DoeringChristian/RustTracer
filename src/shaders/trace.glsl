#version 450

struct Vert{
    vec4 pos;
    vec4 color;
};
struct BVHNode{
    vec4 min;
    vec4 max;
    uint ty;
    uint right;
    uint miss;
};

layout(set = 0, binding = 0) buffer BVH{
    BVHNode nodes[];
}bvh;
layout(set = 0, binding = 1) buffer Verts{
    Vert verts[];
};
layout(set = 0, binding = 2) buffer Indices{
    uint indices[];
};

layout(set = 1, binding = 0, rgba8) writeonly uniform image2D dst;

const float PI = 3.14159265358979323846264338327950288;

float rand2(vec2 co){
    highp float a = 12.9898;
    highp float b = 78.233;
    highp float c = 43758.5453;
    highp float dt= dot(co.xy ,vec2(a,b));
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

struct Ray{
    vec4 pos;
    vec4 dir;
};

struct ClosestHitReturn{
    Ray ray;
    vec4 color;
};


void main(){
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    imageStore(dst, ivec2(x, y), vec4(rand2(vec2(float(x), float(y))), 0.0, 0.0, 1.0));
    //imageStore(dst, ivec2(x, y), vec4(1., 0., 0., 1.));
}
