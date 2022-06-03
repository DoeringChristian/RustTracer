use std::borrow::Borrow;
use std::{fmt::Debug, borrow::BorrowMut};
use std::path::Path;
use std::ops::{Deref, DerefMut};

use archery::*;
use screen_13::prelude_arc::*;

use crate::{
    aabb::IndexedAABB,
    glsl_bvh::{GlslBVH, GlslBVHNode},
    model::Model,
};

pub struct WorldNode {
    pub vertices: Vec<BufferLeaseNode>,
    pub indices: Vec<BufferLeaseNode>,
    pub blas: Vec<BufferLeaseNode>,
    pub tlas: BufferLeaseNode,
}

impl WorldNode{
    pub fn unbind(self, rgraph: &mut RenderGraph) -> WorldBinding{
        let Self{
            vertices,
            indices,
            blas,
            tlas,
        } = self;
        let vertices = vertices.into_iter().map(|v| rgraph.unbind_node(v)).collect::<Vec<_>>();
        let indices = indices.into_iter().map(|i| rgraph.unbind_node(i)).collect::<Vec<_>>();
        let blas = blas.into_iter().map(|b| rgraph.unbind_node(b)).collect::<Vec<_>>();
        let tlas = rgraph.unbind_node(tlas);
        WorldBinding{
            vertices,
            indices,
            blas,
            tlas,
        }
    }

    pub fn record_descriptors<'a>(&self, mut pass_ref: PipelinePassRef<'a, ComputePipeline>) -> PipelinePassRef<'a, ComputePipeline>{
        for (i, blas) in self.blas.iter().enumerate(){
            pass_ref = pass_ref.read_descriptor((0, 0, [i as u32]), *blas);
        }
        for (i, vertices) in self.vertices.iter().enumerate(){
            pass_ref = pass_ref.read_descriptor((0, 1, [i as u32]), *vertices);
        }
        for (i, indices) in self.indices.iter().enumerate(){
            pass_ref = pass_ref.read_descriptor((0, 2, [i as u32]), *indices);
        }
        pass_ref = pass_ref.read_descriptor((0, 3), self.tlas);
        pass_ref
    }
}

pub struct WorldBinding {
    pub vertices: Vec<BufferLeaseBinding<ArcK>>,
    pub indices: Vec<BufferLeaseBinding<ArcK>>,
    // Bottom level acceleration structure.
    pub blas: Vec<BufferLeaseBinding<ArcK>>,
    pub tlas: BufferLeaseBinding<ArcK>,
}

impl WorldBinding{
    pub fn bind(self, rgraph: &mut RenderGraph) -> WorldNode{
        let Self{
            vertices,
            indices,
            blas,
            tlas,
        } = self;
        let vertices = vertices.into_iter().map(|v| rgraph.bind_node(v)).collect::<Vec<_>>();
        let indices = indices.into_iter().map(|i| rgraph.bind_node(i)).collect::<Vec<_>>();
        let blas = blas.into_iter().map(|b| rgraph.bind_node(b)).collect::<Vec<_>>();
        let tlas = rgraph.bind_node(tlas);
        WorldNode{
            vertices,
            indices,
            blas,
            tlas,
        }
    }
}

pub struct RefMut<'w>{
    world: &'w mut World,
}

impl<'w> Deref for RefMut<'w>{
    type Target = World;

    fn deref(&self) -> &Self::Target {
        self.world
    }
}

impl<'w> DerefMut for RefMut<'w>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.world
    }
}

impl<'w> Drop for RefMut<'w>{
    fn drop(&mut self) {
        self.world.create_bvh()
    }
}

pub struct World {
    pub models: Vec<Model>,
    pub bvh: Option<GlslBVH>,
}

impl World{
    pub fn borrow_mut(&mut self) -> RefMut{
        RefMut{world: self}
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            models: Vec::default(),
            bvh: None,
        }
    }
    pub fn append_obj(&mut self, path: impl AsRef<Path> + Debug) {
        let models = tobj::load_obj(path, &tobj::LoadOptions::default())
            .unwrap()
            .0;
        self.models = models.iter().map(|m| Model::from_obj_model(m)).collect();
        self.create_bvh();
    }
    pub fn create_bvh(&mut self) {
        self.bvh = Some(GlslBVH::build_buckets_16(
            self.models.iter().enumerate().map(|(i, _)| IndexedAABB {
                index: i,
                aabb: self.models[i].aabb(),
            }),
        ))
    }
    pub fn upload(&self, cache: &mut HashPool) -> WorldBinding {
        let vertices = self
            .models
            .iter()
            .map(|m| m.upload_verts(cache))
            .collect::<Vec<_>>();
        let indices = self
            .models
            .iter()
            .map(|m| m.upload_indices(cache))
            .collect::<Vec<_>>();
        let blas = self
            .models
            .iter()
            .map(|m| m.upload_bvh(cache))
            .collect::<Vec<_>>();
        let tlas = BufferLeaseBinding({
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
        });

        WorldBinding {
            vertices,
            indices,
            blas,
            tlas,
        }
    }
}
