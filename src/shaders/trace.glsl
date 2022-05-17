#version 450

struct Vert{
    vec4 pos;
    vec4 color;
};
#define TY_NODE 0
#define TY_LEAF 1
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

struct RayReturn{
    Ray ray;
    vec4 color;
};

bool intersects_aabb(Ray ray, vec4 min, vec4 max){
    // TODO: intersection.
    float t_near = (min.x - ray.pos.x)/ray.dir.x;
    t_near = max(t_near, (min.y - ray.pos.y)/ray.dir.y);
    t_near = max(t_near, (min.z - ray.pos.z)/ray.dir.z);

    float t_far = (max.x - ray.pos.x)/ray.dir.x;
    t_far = min(t_far, (max.y - ray.pos.y)/ray.dir.y);
    t_far = min(t_far, (max.z - ray.pos.z)/ray.dir.z);

    if(t_near > t_far || t_far < 0)
        return false;
    return true;
}

RayReturn closest_hit(Vert hit, Ray ray, uint blas_id){
    return RayReturn(ray, vec4(1., 0., 0., 1.));
}

vec4 miss(Ray ray){
    return vec4(0., 1., 0., 1.);
}

Vert intersection(Ray ray, uint blas_id, uint index_id){
}

Ray ray_gen(vec2 ss, uint ray_num){
    return Ray(vec4(1., 0., 0., 1.), vec4(-1., 0., 0., 1.));
}

bool anyhit(Vert hit, uint blas_id){
    return true;
}


void main(){
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    uint z = gl_GlobalInvocationID.z;

    Ray ray = ray_gen(vec2(float(x), float(y)), z);

    // Start at root node.
    uint blas_id = 0;
    while(true){
        BVHNode node = bvh.nodes[blas_id];
        if(intersects_aabb(ray, node.min, node.max)){
            if(node.ty == TY_NODE){
                // Traverse left nodes
                blas_id++;
            }
            else if(node.ty == TY_LEAF){
                Vert v0 = verts[indices[node.right+0]];
                Vert v1 = verts[indices[node.right+1]];
                Vert v2 = verts[indices[node.right+2]];
            }
        }
        else{
            // If we missed the aabb with the ray we go to the miss node.
            blas_id = node.miss;
        }
        break;
    }

    imageStore(dst, ivec2(x, y), vec4(rand2(vec2(float(x), float(y))), 0.0, 0.0, 1.0));
    //imageStore(dst, ivec2(x, y), vec4(1., 0., 0., 1.));
}
