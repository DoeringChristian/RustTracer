#version 450
//#extension GL_EXT_nonunifomr_qualifier: require
#if COMPUTE_SHADER


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

void main(){
    imageStore(dst, ivec2(1, 1), vec4(1.0, 0.0, 0.0, 1.0));
}

#endif
