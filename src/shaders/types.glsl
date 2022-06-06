#ifndef TYPES_GLSL
#define TYPES_GLSL

struct Vert{
    vec4 pos;
    vec4 color;
    vec4 normal;
    uint has_mat;
    uint mat_idx;
};

const Vert vert_default = Vert(vec4(0., 0., 0., 1.), vec4(0., 0., 0., 1.), vec4(0., 0., 0., 1.), 0, 0);

struct Material{
    vec4 color;
};



// internal types.

struct Intersection{
    Vert vert;
    vec3 uvt;
    //uint blas_id;
    //uint index_id;
    bool intersected;
};

struct Ray{
    vec4 pos;
    vec4 dir;
};

struct RayPayload{
    Ray ray;
    vec4 color;
    float refl;
};

#define TY_NODE 0
#define TY_LEAF 1
struct BVHNode{
    vec4 min;
    vec4 max;
    uint ty;
    uint right;
    uint miss;
    uint _pad;
};

#endif
