#ifndef PBR_UTILS_GLSL
#define PBR_UTILS_GLSL

#include "utils.glsl"


vec3 rand_ball(vec3 seed){
    uint c = 0;
    vec3 r = rand3(seed);
    while(c < 5){
        if (length(r) <= 1.0){
            return r;
        }
        vec3 r = rand3(seed);
        c++;
    }
    return r;
}

vec3 rand_sphere(vec3 seed){
    return normalize(rand_ball(seed));
}

vec3 rand_halfsphere(vec3 seed, vec3 n){
    vec3 r = rand_sphere(seed);
    if (dot(r, n) < 0.){
        return reflect(r, n);
    }
    return r;
}

// from learnopengl
float DistributionGGX(vec3 N, vec3 H, float a)
{
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float nom    = a2;
    float denom  = (NdotH2 * (a2 - 1.0) + 1.0);
    denom        = PI * denom * denom;
	
    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float k)
{
    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return nom / denom;
}
  
float GeometrySmith(vec3 N, vec3 V, vec3 L, float k)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx1 = GeometrySchlickGGX(NdotV, k);
    float ggx2 = GeometrySchlickGGX(NdotL, k);
	
    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

#endif
