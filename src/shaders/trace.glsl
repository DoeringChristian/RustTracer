#version 460 core
#extension GL_EXT_nonuniform_qualifier : require

#include "pbr.glsl"
#include "interface.glsl"
#include "types.glsl"
#include "utils.glsl"

//============================================================
// Main function for managing ray tracing shaders:
//============================================================

const uint RAY_COUNT = 1;
const uint BVH_LIMIT = 2000;

void main(){
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;
    uint z = gl_GlobalInvocationID.z;
    imageStore(dst, ivec2(x, y), vec4(0., 0., 0., 1.));

    RayPayload ray = ray_gen(vec2(float(x), float(y)), z);
    uint ray_num = 0;
    uint bvh_count = 0;
    uint blas_id = 0;

    while(ray_num < num_paths){
        Intersection closest_inter = Intersection(vert_default, vec3(0., 0., 1./0.), false);
        // Start at root node of tlas.
        uint tnode_idx = 0;
        //==============================
        // Start Traverse tlas:
        //==============================
        while(true){
            BVHNode tnode = tlas.nodes[tnode_idx];
            // Test for intersections in the tlas
            if(intersects_aabb(ray.ray, tnode.min, tnode.max)){
                if(tnode.ty == TY_NODE){
                    // left most nodes are at i+1 because the tree is safed in pre order
                    tnode_idx ++;
                }
            else if (tnode.ty == TY_LEAF){
                    // We have hit a leaf of the tlas therefore iterate throught the blas accociated with that index.

                    //==============================
                    // Start Traverse blas:
                    //==============================

                    uint bnode_idx = 0;
                    while(true){
                        BVHNode bnode = blas[tnode.right].nodes[bnode_idx];
                        if(intersects_aabb(ray.ray, bnode.min, bnode.max)){
                            if(bnode.ty == TY_NODE){
                                bnode_idx++;
                            }
                        else if(bnode.ty == TY_LEAF){
                                Intersection inter = intersection(ray.ray, tnode.right, bnode.right);
                                if(inter.intersected && inter.uvt.z < closest_inter.uvt.z && anyhit(inter)){
                                    closest_inter = inter;
                                }

                                // Break if we missed and the bnode is a right most bnode.
                                if (bnode.miss == 0){
                                    break;
                                }

                                // We have missed the shape and go to the miss node.
                                bnode_idx = bnode.miss;
                            }
                        }
                    else{
                            // Break if we missed and the bnode is a right most bnode.
                            if(bnode.miss == 0){
                                break;
                            }

                            // We have missed the aabb and go to the miss-node.
                            bnode_idx = bnode.miss;
                        }
                    }
                    //==============================
                    // End Traverse blas:
                    //==============================

                    // Break if we missed and the bnode is a right most tnode.
                    if (tnode.miss == 0){
                        break;
                    }

                    tnode_idx = tnode.miss; 
                }
            }
        else{
                // Break if we missed and the bnode is a right most tnode.
                if(tnode.miss == 0){
                    break;
                }

                tnode_idx = tnode.miss;
            }
        }
        //==============================
        // End Traverse tlas:
        //==============================
        if (closest_inter.intersected == true){
            ray = closest_hit(closest_inter.vert, ray);
        }
    else{
            // There has not been any hit so we return the miss color.
            ray = miss(ray);
            break;
        }
        ray_num++;
    }

    //imageAtomicAdd(dst, ivec2(x, y), vec4(ray.color)/float(num_paths))
    imageStore(dst, ivec2(x, y), vec4(ray.color, 1.));
}
