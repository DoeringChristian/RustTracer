#ifndef PBR_GLSL
#define PBR_GLSL

#include "pbr_utils.glsl"
#include "interface.glsl"
#include "types.glsl"
#include "utils.glsl"

//============================================================
// Ray tracing shader:
//============================================================

//===============================
// Intersection Shader:
//===============================
// This shader calculates the intersection between a ray and an object in this case a triangle.
// It also calculates the interpolated hit fragment.
Intersection intersection(Ray ray, uint blas_id, uint index_id){
    Vert v0 = tverts[blas_id].verts[tindices[blas_id].indices[index_id + 0]];
    Vert v1 = tverts[blas_id].verts[tindices[blas_id].indices[index_id + 1]];
    Vert v2 = tverts[blas_id].verts[tindices[blas_id].indices[index_id + 2]];
    vec3 uvt = triangle_uvt(ray, v0.pos.xyz, v1.pos.xyz, v2.pos.xyz);
    if(uvt.x + uvt.y <= 1 && uvt.x >= 0 && uvt.y >= 0){
        vec4 pos_int = mix2(v0.pos, v1.pos, v2.pos, uvt.x, uvt.y);
        vec4 color_int = mix2(v0.color, v1.color, v2.color, uvt.x, uvt.y);
        // generate normals.
        //vec4 normal_int = mix2(v0.normal, v1.normal, v2.normal, uvt.x, uvt.y);
        vec4 normal_int = vec4(normalize(cross(v0.pos.xyz - v1.pos.xyz, v0.pos.xyz - v2.pos.xyz)), 1.);
        Vert vert_int = Vert(pos_int, color_int, normal_int, v0.has_mat, v0.mat_idx);
        return Intersection(vert_int, uvt, true);
    }
    else{
        return Intersection(v0, uvt, false);
    }
}

//===============================
// Ray Generation:
//===============================
// This shader generates the initial rays cast from the camera.
RayPayload ray_gen(vec2 screen_pos, uint ray_num){
    uint z = gl_GlobalInvocationID.z;
    // Offset at random in range (-0.5, 0.5)
    screen_pos += vec2(rand(float(z)), rand(float(z+10000))) - vec2(0.5, 0.5);
    // Normalize to screen width / height.
    screen_pos /= vec2(float(width), float(height));
    // Offset to center
    screen_pos -= vec2(0.5, 0.5);
    Ray ray = Ray(vec3(0., 3., 0.), vec3(screen_pos.x, -1., screen_pos.y));
    return RayPayload(ray, vec3(1., 1., 1.), vec3(1., 1., 1.));
}

//===============================
// Closest Hit shader:
//===============================
// This shader is called on the fragment hit closest to the camera.
// The main shader code resides here.
RayPayload closest_hit(Vert hit, RayPayload prev){
    vec3 n = hit.normal.xyz;
    vec3 v = -prev.ray.dir.xyz;
    vec3 ray_dir = rand_halfsphere(hit.pos.xyz, n);
    vec3 ray_pos = prev.ray.pos.xyz;

    float metalness = 0.5;
    vec3 surface_color;
    if (hit.has_mat == 1){
        surface_color = materials[hit.mat_idx].color.rgb;
    } else{
        surface_color = vec3(1., 0., 0.);
    }

    float a = 0.5;
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, surface_color.rgb, metalness);

    vec3 h = normalize(v + ray_dir.xyz);
    float cosTheta = dot(h, n);

    float D = DistributionGGX(n, h, a);

    vec3 F = fresnelSchlick(cosTheta, F0);
    vec3 ks = F;
    vec3 kd = 1-ks;

    //float k_ibl = (a*a)/2.;
    float G = GeometrySmith(n, v, ray_dir.xyz, metalness);

    float win = dot(ray_dir, n);
    float won = dot(prev.ray.dir.xyz, n);

    vec3 specular = ks * D*G*F / (4 * win * won);
    vec3 brdf = (kd * surface_color/PI + specular) * win;

    float prev_ray_len = length(prev.ray.pos.xyz - hit.pos.xyz);
    float att = 1/(prev_ray_len * prev_ray_len);
    return RayPayload(
        Ray(
            vec3(ray_pos),
            vec3(ray_dir)
        ),
        prev.color * att * brdf,
        vec3(0.)
    );
}

//===============================
// Miss shader:
//===============================
// If there have been no further hits (the ray goes into the void) this shader is called.
// The color of the output from this shader is the color displayed on screen.
RayPayload miss(RayPayload prev){
    return RayPayload(prev.ray, prev.color * vec3(prev.ray.dir.x, prev.ray.dir.y, prev.ray.dir.z) * 500., vec3(0.));
}

//===============================
// Any Hit shader:
//===============================
// If there is a transparent part of a shape this shader can allow the ray to pass through.
bool anyhit(Intersection inter){
    return true;
}

#endif
