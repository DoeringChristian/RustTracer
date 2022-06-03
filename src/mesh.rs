
use std::{path::Path, fmt::Debug};
use crate::{aabb::*, bvh::{BVHNode, BVH}, glsl_bvh::GlslBVH};
use screen_13::prelude_arc::*;
use archery::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl From<[Vert; 3]> for AABB {
    fn from(src: [Vert; 3]) -> Self {
        let v1 = src[0].pos;
        let v2 = src[1].pos;
        let v3 = src[2].pos;
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
            self.verts[self.indices[index + 0] as usize],
            self.verts[self.indices[index + 1] as usize],
            self.verts[self.indices[index + 2] as usize],
        ]
    }
    pub fn get_for_tri(&self, indices: &[usize; 3]) -> [Vert; 3] {
        [
            self.verts[indices[0]],
            self.verts[indices[1]],
            self.verts[indices[2]],
        ]
    }

    pub fn from_obj_mesh(mesh: &tobj::Mesh) -> Self{
        let verts = (0..(mesh.positions.len() / 3))
            .into_iter()
            .map(|i| Vert {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    0.,
                ],
                color: [
                    *mesh.vertex_color.get(i * 3).unwrap_or(&0.),
                    *mesh.vertex_color.get(i * 3 + 1).unwrap_or(&0.),
                    *mesh.vertex_color.get(i * 3 + 2).unwrap_or(&0.),
                    1.,
                ],
            })
        .collect();

        let indices = (0..(mesh.indices.len()))
            .into_iter()
            .map(|i| mesh.indices[i] as u32)
            .collect();
        Self{
            verts,
            indices,
        }
    }

    pub fn load_obj(path: impl AsRef<Path> + Debug) -> Self{
        let models = tobj::load_obj(path, &tobj::LoadOptions::default()).unwrap().0;
        Self::from_obj_mesh(&models[0].mesh)
    }

    pub fn create_bvh_glsl(&self) -> GlslBVH{
        GlslBVH::build_buckets_16(
            (0..self.indices.len() / 3)
            .into_iter()
            .map(|i| IndexedAABB{
                index: i * 3,
                aabb: self.get_tri(i * 3).into(),
            }),
        )
    }

    pub fn upload_verts(&self, cache: &mut HashPool) -> BufferLeaseBinding<ArcK>{
        BufferLeaseBinding({
            let mut buf = cache
                .lease(BufferInfo::new_mappable(
                        (std::mem::size_of::<Vert>() * self.verts.len()) as u64,
                        vk::BufferUsageFlags::STORAGE_BUFFER,
                )).unwrap();
            Buffer::copy_from_slice(buf.get_mut().unwrap(), 0, bytemuck::cast_slice(&self.verts));
            buf
        })
    }

    pub fn upload_indices(&self, cache: &mut HashPool) -> BufferLeaseBinding<ArcK>{
        BufferLeaseBinding({
            let mut buf = cache
                .lease(BufferInfo::new_mappable(
                        (std::mem::size_of::<u32>() * self.indices.len()) as u64,
                        vk::BufferUsageFlags::STORAGE_BUFFER,
                ))
                .expect("Could not create Index Buffer");
            Buffer::copy_from_slice(
                buf.get_mut().expect("Could not get Index Buffer"),
                0,
                bytemuck::cast_slice(&self.indices),
            );
            buf
        })
    }
}
