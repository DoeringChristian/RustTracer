use std::ops::Range;

mod aabb;
mod bvh;
mod glsl_bvh;

use aabb::*;
use bvh::*;
use glsl_bvh::*;

pub trait Pos3 {
    fn pos3(&self) -> [f32; 3];
}

#[derive(Copy, Clone)]
pub struct Vert {
    pub pos: [f32; 4],
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
    pub tris: Vec<[usize; 3]>,
}

/*
impl BvhShape for Mesh{
    fn primitive_iter(&self) -> dyn Iterator<Item = (usize, AABB)> {
        self.tris.iter().enumerate().map(|(i, tri)|{(i, self.get_tri(i).into())})
    }
}
*/

impl Mesh {
    pub fn get_tri(&self, index: usize) -> [Vert; 3] {
        [
            self.verts[self.tris[index][0]],
            self.verts[self.tris[index][1]],
            self.verts[self.tris[index][2]],
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
        },
        Vert {
            pos: [1., 0., 0., 0.],
        },
        Vert {
            pos: [1., 1., 0., 0.],
        },
        Vert {
            pos: [2., 0., 0., 0.],
        },
        Vert {
            pos: [3., 0., 0., 0.],
        },
        Vert {
            pos: [3., 1., 0., 0.],
        },
        Vert {
            pos: [2., 1., 0., 0.],
        },
        Vert {
            pos: [3., 1., 0., 0.],
        },
        Vert {
            pos: [3., 2., 0., 0.],
        },
    ];
    let tris = vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]];

    let mesh = Mesh { verts, tris };

    let bvh = GlslBVH::build_sweep(
        mesh.tris
            .iter()
            .enumerate()
            .map(|(i, tri)| (i, mesh.get_for_tri(tri).into())),
    );
    bvh.print_rec(0, &mut String::from(""));

    let suzanne = tobj::load_obj("src/assets/suzanne.obj", &tobj::LoadOptions::default()).unwrap().0;

    let verts = (0..(suzanne[0].mesh.positions.len()/3)).into_iter()
            .map(|i|{
                Vert{pos: [
                    suzanne[0].mesh.positions[i*3], 
                    suzanne[0].mesh.positions[i*3+1], 
                    suzanne[0].mesh.positions[i*3+2], 
                    0.
                ]}
            }).collect();

    let tris = (0..(suzanne[0].mesh.indices.len()/3)).into_iter()
        .map(|i|{
            [
                suzanne[0].mesh.indices[i*3] as usize,
                suzanne[0].mesh.indices[i*3+1] as usize,
                suzanne[0].mesh.indices[i*3+2] as usize,
            ]
        }).collect();

    let mesh = Mesh{
        verts,
        tris,
    };

    let bvh = GlslBVH::build_buckets_16(
        mesh.tris
            .iter()
            .enumerate()
            .map(|(i, tri)| (i, mesh.get_for_tri(tri).into())),
    );
}
