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
    if(uvt.x + uvt.y <= 1 && uvt.x >= 0 && uvt.y >= 0 && uvt.z > 0){
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
    Ray ray = Ray(vec3(0., -3., 0.), vec3(screen_pos.x, 1., screen_pos.y));
    return RayPayload(ray, vec3(1., 1., 1.), vec3(1., 1., 1.));
}

//===============================
// Closest Hit shader:
//===============================
// This shader is called on the fragment hit closest to the camera.
// The main shader code resides here.
RayPayload closest_hit(Vert hit, RayPayload prev){

    float metallic = 0.9;
    float roughness = 0.1;

    vec3 n = hit.normal.xyz;
    vec3 v = normalize(-prev.ray.dir.xyz);
    vec3 l = rand_halfsphere(hit.pos.xyz, n);
    vec3 h = normalize(v + l);
    float prev_len = length(prev.ray.pos.xyz - hit.pos.xyz);
    float vatt = 1./(prev_len * prev_len); // Attenuation to the view point

    vec3 ray_dir = l;
    vec3 ray_pos = hit.pos.xyz + l * 0.01;

    vec3 albedo;
    if (hit.has_mat == 1){
        albedo = materials[hit.mat_idx].color.rgb;
    } else{
        albedo = vec3(0., 1., 0.);
    }

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    float D = DistributionGGX(n, h, roughness);

    vec3 F = fresnelSchlick(clamp(dot(h, v), 0., 1.), F0);

    float G = GeometrySmith(n, v, l, roughness);

    float win = dot(l, n);
    float won = dot(v, n);

    vec3 ks = F;
    vec3 kd = vec3(1.)-ks;
    kd *= 1.0 - metallic;

    vec3 specular = D*G*F / (4. * max(win, 0.) * max(won, 0.) + 0.0001);
    vec3 brdf = (kd * albedo/PI + specular);

    return RayPayload(
        Ray(
            vec3(ray_pos),
            vec3(ray_dir)
        ),
        prev.color * vatt * brdf * win,
        vec3(0.)
    );
}

//===============================
// Miss shader:
//===============================
// If there have been no further hits (the ray goes into the void) this shader is called.
// The color of the output from this shader is the color displayed on screen.
RayPayload miss(RayPayload prev){
    vec3 light_color;
    if (dot(normalize(prev.ray.dir), vec3(1., 0., 0.)) > 0.1){
        light_color = vec3(1.);
    } else{
        light_color = vec3(0.);
    }
    return RayPayload(
        prev.ray, 
        prev.color * light_color * 100., 
        vec3(0.));
}

//===============================
// Any Hit shader:
//===============================
// If there is a transparent part of a shape this shader can allow the ray to pass through.
bool anyhit(Intersection inter){
    return true;
}

#endif
