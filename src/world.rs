use std::fmt::Debug;
use std::path::Path;

use screen_13::prelude_arc::*;
use archery::*;

use crate::{model::Model, glsl_bvh::GlslBVH, aabb::IndexedAABB};

pub struct GpuWorld{
    pub vertices: Vec<Option<BufferLeaseBinding<ArcK>>>,
    pub indices: Vec<Option<BufferLeaseBinding<ArcK>>>,
    // Bottom level acceleration structure.
    pub blas: Vec<Option<BufferLeaseBinding<ArcK>>>,
    pub tlas: Option<BufferLeaseBinding<ArcK>>,
}

pub struct World{
    pub models: Vec<Model>,
    pub bvh: Option<GlslBVH>,
}

impl World{
    pub fn new() -> Self{
        Self{
            models: Vec::default(),
            bvh: None,
        }
    }
    pub fn append_obj(&mut self, path: impl AsRef<Path> + Debug){
        let models = tobj::load_obj(path, &tobj::LoadOptions::default()).unwrap().0;
        self.models = models.iter().map(|m| Model::from_obj_model(m)).collect();
        self.create_bvh();
    }
    pub fn create_bvh(&mut self){
        self.bvh = Some(GlslBVH::build_buckets_16(
                self.models.iter().enumerate().map(|(i, _)|{
                    IndexedAABB{
                        index: i,
                        aabb: self.models[i].aabb(),
                    }
                })
        ))
    }
}
