use std::fmt::Debug;
use std::path::Path;

use crate::aabb::AABB;
use crate::glsl_bvh::GlslBVHNode;
use crate::mesh::*;
use crate::GlslBVH;
use archery::*;
use screen_13::prelude_arc::*;

pub struct Model {
    mesh: Mesh,
    // transfomrs
    bvh: Option<GlslBVH>,
}

impl Model {
    pub fn load_obj(path: impl AsRef<Path> + Debug) -> Self {
        let models = tobj::load_obj(path, &tobj::LoadOptions::default())
            .unwrap()
            .0;
        Self::from_obj_model(&models[0])
    }
    pub fn from_obj_model(model: &tobj::Model) -> Self {
        let mesh = Mesh::from_obj_mesh(&model.mesh);
        let bvh = Some(mesh.create_bvh_glsl());
        Self { mesh, bvh }
    }
    pub fn create_bvh(&mut self) {
        self.bvh = Some(self.mesh.create_bvh_glsl());
    }

    pub fn aabb(&self) -> AABB {
        self.bvh.as_ref().unwrap().aabb()
    }

    pub fn upload_verts(&self, cache: &mut HashPool) -> BufferLeaseBinding<ArcK> {
        self.mesh.upload_verts(cache)
    }
    pub fn upload_indices(&self, cache: &mut HashPool) -> BufferLeaseBinding<ArcK> {
        self.mesh.upload_indices(cache)
    }
    pub fn upload_bvh(&self, cache: &mut HashPool) -> BufferLeaseBinding<ArcK> {
        BufferLeaseBinding({
            let mut buf = cache
                .lease(BufferInfo::new_mappable(
                        (std::mem::size_of::<GlslBVHNode>() * self.bvh.as_ref().unwrap().nodes().len())
                        as u64,
                        vk::BufferUsageFlags::STORAGE_BUFFER,
                ))
                .unwrap();
            Buffer::copy_from_slice(
                buf.get_mut().unwrap(),
                0,
                bytemuck::cast_slice(self.bvh.as_ref().unwrap().nodes()),
            );
            buf
        })
    }
}
