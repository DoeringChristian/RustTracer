use std::ops::Range;

mod aabb;
mod bvh;
mod glsl_bvh;

use aabb::*;
use bvh::*;
use glsl_bvh::*;
use ewgpu::*;

pub trait Pos3 {
    fn pos3(&self) -> [f32; 3];
}

#[derive(Copy, Clone)]
pub struct Vert {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl Pos3 for Vert {
    fn pos3(&self) -> [f32; 3] {
        [self.pos[0], self.pos[1], self.pos[2]]
    }
}

impl From<[Vert; 3]> for AABB {
    fn from(src: [Vert; 3]) -> Self {
        let v1 = src[0].pos3();
        let v2 = src[1].pos3();
        let v3 = src[2].pos3();
        AABB {
            min: [
                v1[0].min(v2[0]).min(v3[0]),
                v1[1].min(v2[1]).min(v3[1]),
                v1[2].min(v2[2]).min(v3[2]),
            ],
            max: [
                v1[0].max(v2[0]).max(v3[0]),
                v1[1].max(v2[1]).max(v3[1]),
                v1[2].max(v2[2]).max(v3[2]),
            ],
        }
    }
}

pub struct Mesh {
    pub verts: Vec<Vert>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn get_tri(&self, index: usize) -> [Vert; 3] {
        [
            self.verts[self.indices[index] as usize],
            self.verts[self.indices[index] as usize],
            self.verts[self.indices[index] as usize],
        ]
    }
    pub fn get_for_tri(&self, indices: &[usize; 3]) -> [Vert; 3] {
        [
            self.verts[indices[0]],
            self.verts[indices[1]],
            self.verts[indices[2]],
        ]
    }
}

fn main() {
    let verts = vec![
        Vert {
            pos: [0., 0., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [1., 0., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [1., 1., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [2., 0., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [3., 0., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [3., 1., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [2., 1., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [3., 1., 0., 0.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [3., 2., 0., 0.],
            color: [0., 0., 0., 1.],
        },
    ];
    let indices = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];

    let mesh = Mesh { verts, indices };

    let bvh = GlslBVH::build_sweep(
        (0..mesh.indices.len() / 3)
            .into_iter()
            .map(|i| IndexedAABB{ index: i * 3, aabb: mesh.get_tri(i * 3).into()}),
    );
    bvh.print_rec(0, &mut String::from(""));

    let suzanne = tobj::load_obj("src/assets/suzanne.obj", &tobj::LoadOptions::default())
        .unwrap()
        .0;

    let verts = (0..(suzanne[0].mesh.positions.len() / 3))
        .into_iter()
        .map(|i| Vert {
            pos: [
                suzanne[0].mesh.positions[i * 3],
                suzanne[0].mesh.positions[i * 3 + 1],
                suzanne[0].mesh.positions[i * 3 + 2],
                0.,
            ],
            color: [
                *suzanne[0].mesh.vertex_color.get(i * 3).unwrap_or(&0.),
                *suzanne[0].mesh.vertex_color.get(i * 3 + 1).unwrap_or(&0.),
                *suzanne[0].mesh.vertex_color.get(i * 3 + 2).unwrap_or(&0.),
                1.,
            ],
        })
        .collect();

    let indices = (0..(suzanne[0].mesh.indices.len()))
        .into_iter()
        .map(|i| suzanne[0].mesh.indices[i] as u32)
        .collect();

    let mesh = Mesh { verts, indices };

    let bvh = GlslBVH::build_buckets_16(
        (0..mesh.indices.len() / 3)
            .into_iter()
            .map(|i| IndexedAABB{ index: i * 3, aabb: mesh.get_tri(i * 3).into()}),
    );
    //bvh.print_rec(0, &mut String::from(""));

    let mut gpu = GPUContextBuilder::new()
        .set_features_util()
        .set_limits(wgpu::Limits{
            max_push_constant_size: 128,
            ..Default::default()
        }).build();

    gpu.encode_img([800, 600], |gpu, view, encoder|{

    });
}
