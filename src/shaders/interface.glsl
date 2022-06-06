#ifndef INTERFACE_GLSL
#define INTERFACE_GLSL

#include "types.glsl"

layout(set = 0, binding = 0) buffer BLAS{
    BVHNode nodes[];
}blas[];
layout(set = 0, binding = 1) buffer Verts{
    Vert verts[];
}tverts[];
layout(set = 0, binding = 2) buffer Indices{
    uint indices[];
}tindices[];
layout(set = 0, binding = 3) buffer TLAS{
    BVHNode nodes[];
}tlas;

layout(set = 0, binding = 4) buffer MaterialBlock{
    Material materials[];
};

layout(push_constant) uniform PushConstants{
    uint width;
    uint height;
    uint num_paths;
};

layout(set = 1, binding = 0, rgba8) writeonly uniform image2D dst;

#endif
