
struct Vert{
    pos: [[stride(4)]] vec4<f32>;
    color: [[stride(4)]] vec4<f32>;
};

struct BVHNode{
    min: vec4<f32>;
    max: vec4<f32>;
    ty: u32;
    right: u32;
    miss: u32;
    _pad: u32;
};

[[group(0), binding(0)]]
var<storage, read_write> bvh: array<BVHNode>;
[[group(0), binding(1)]]
var<storage, read_write> verts: array<Vert>;
[[group(0), binding(2)]]
var<storage, read_write> indices: array<u32>;

[[group(1), binding(0)]]
var<storage, write> dst: texture_storage_2d<rgba8unorm, write>;

[[stage(compute), workgroup_size(1)]]
fn cs_main([[builtin(global_invocation_id)]] global_id: vec3<u32>){

}

