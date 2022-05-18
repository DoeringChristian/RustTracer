#version 450

struct Vert{
    vec4 pos;
    vec4 color;
};

const Vert vert_default = Vert(vec4(0., 0., 0., 1.), vec4(0., 0., 0., 1.));

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

struct Intersection{
    Vert vert;
    vec3 uvt;
    uint blas_id;
    bool intersected;
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

struct RayPayload{
    Ray ray;
    vec4 color;
    float refl;
};

bool intersects_aabb(Ray ray, vec4 bmin, vec4 bmax){
    // TODO: intersection.
    float t_near = (bmin.x - ray.pos.x)/ray.dir.x;
    t_near = max(t_near, (bmin.y - ray.pos.y)/ray.dir.y);
    t_near = max(t_near, (bmin.z - ray.pos.z)/ray.dir.z);

    float t_far = (bmax.x - ray.pos.x)/ray.dir.x;
    t_far = min(t_far, (bmax.y - ray.pos.y)/ray.dir.y);
    t_far = min(t_far, (bmax.z - ray.pos.z)/ray.dir.z);

    // DEBUG:
    return true;

    if(t_near > t_far || t_far < 0)
        return false;
    return true;
}

RayPayload closest_hit(Vert hit, RayPayload ray, uint blas_id){

    return RayPayload(ray.ray, vec4(1., 0., 0., 1.), 0.0);
    //return RayPayload(ray.ray, ray.color + vec4(1., 0., 0., 1.) * ray.refl, ray.refl * 0.1);
}

RayPayload miss(RayPayload ray){
    return RayPayload(ray.ray, vec4(0., 1., 0., 1.), 0.);
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

Intersection intersection(Ray ray, uint blas_id, uint index_id){
    Vert v0 = verts[indices[index_id + 0]];
    Vert v1 = verts[indices[index_id + 1]];
    Vert v2 = verts[indices[index_id + 2]];
    vec3 uvt = triangle_uvt(ray, v0.pos.xyz, v1.pos.xyz, v2.pos.xyz);
    if(uvt.x + uvt.y <= 1 && uvt.x >= 0 && uvt.y >= 0){
        vec4 pos_int = mix2(v0.pos, v1.pos, v2.pos, uvt.x, uvt.y);
        vec4 color_int = mix2(v0.color, v1.color, v2.color, uvt.x, uvt.y);
        Vert vert_int = Vert(pos_int, color_int);
        return Intersection(vert_int, uvt, blas_id, true);
    }
    else{
        return Intersection(v0, uvt, blas_id, false);
    }
}

RayPayload ray_gen(vec2 ss, uint ray_num){
    ss = (ss * 1.)-vec2(0.5, 0.5);
    Ray ray = Ray(vec4(0., 3., 0., 1.), vec4(ss.x, -1., ss.y, 1.));
    return RayPayload(ray, vec4(0., 0., 0., 0.), 1.);
}

bool anyhit(Intersection inter){
    return true;
}

const uint RAY_COUNT = 1;
const uint BVH_LIMIT = 2000;

void main(){
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    uint z = gl_GlobalInvocationID.z;
    imageStore(dst, ivec2(x, y), vec4(0., 0., 0., 1.));

    RayPayload ray = ray_gen(vec2(float(x) / 100., float(y) / 100.), z);
    uint ray_num = 0;
    uint bvh_count = 0;

    while(ray_num < RAY_COUNT){
        // Start at root node.
        uint blas_id = 0;
        Intersection closest_inter = Intersection(vert_default, vec3(0., 0., 1./0.), 0, false);
        while(bvh_count < BVH_LIMIT){
            BVHNode node = bvh.nodes[blas_id];
            if(intersects_aabb(ray.ray, node.min, node.max)){
                if(node.ty == TY_NODE){
                    // Traverse left nodes
                    blas_id++;
                }
                else if(node.ty == TY_LEAF){
                    Intersection inter = intersection(ray.ray, blas_id, node.right);
                    if (inter.intersected && inter.uvt.z < closest_inter.uvt.z && anyhit(inter)){
                        closest_inter = inter;
                    }
                    // Break if we missed and the node is a right most node.
                    if (node.miss == 0){
                        break;
                    }
                    // Goto miss node either way. Because we don't know if ther could be a closer hit.
                    blas_id = node.miss;
                }
            }
            else{
                // break if we have missed and the node is a right most node.
                if (node.miss == 0){
                    break;
                }
                // If we missed the aabb with the ray we go to the miss node.
                blas_id = node.miss;
            }
            bvh_count++;
            // TODO: add Safeguard for the case that blas_id >= blas_num.
            // This should not happen if the bvh has been generated corectly but a check would be good just in case.
        }
        if (closest_inter.intersected == true){
            ray = closest_hit(closest_inter.vert, ray, closest_inter.blas_id);
        }
        else{
            // There has not been any hit so we return the miss color.
            ray = miss(ray);
            break;
        }
        ray_num++;
    }

    imageStore(dst, ivec2(x, y), vec4(ray.color));
}
