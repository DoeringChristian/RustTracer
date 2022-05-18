
use crate::bvh::*;
use crate::aabb::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlslBVHNode {
    pub min: [f32; 4],
    pub max: [f32; 4],
    pub ty: u32,
    pub right: u32,
    pub miss: u32,
    pub _pad: u32,
}
impl GlslBVHNode {
    pub const TY_NODE: u32 = 0x00;
    pub const TY_LEAF: u32 = 0x01;
}
impl BVHNode for GlslBVHNode{
    type ExternIndex = usize;
    #[inline]
    fn new_node(aabb: AABB, right: usize, miss: usize) -> Self {
        GlslBVHNode{
            ty: Self::TY_NODE,
            min: [aabb.min[0], aabb.min[1], aabb.min[2], 0.],
            max: [aabb.max[0], aabb.max[1], aabb.max[2], 0.],
            right: right as u32,
            miss: miss as u32,
            _pad: 0,
        }
    }

    #[inline]
    fn new_leaf(aabb: AABB, index: usize, miss: usize) -> Self {
        GlslBVHNode{
            ty: Self::TY_LEAF,
            min: [aabb.min[0], aabb.min[1], aabb.min[2], 0.],
            max: [aabb.max[0], aabb.max[1], aabb.max[2], 0.],
            right: index as u32,
            miss: miss as u32,
            _pad: 0,
        }
    }

    #[inline]
    fn set_right(&mut self, right: usize) {
        self.right = right as u32;
    }

    #[inline]
    fn set_miss(&mut self, miss: usize) {
        self.miss = miss as u32;
    }

    #[inline]
    fn miss(&self) -> usize {
        self.miss as usize
    }

    #[inline]
    fn right(&self) -> usize {
        self.right as usize
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        self.ty == Self::TY_LEAF
    }

    #[inline]
    fn is_node(&self) -> bool {
        self.ty == Self::TY_NODE
    }
}

pub type GlslBVH = BVH<GlslBVHNode>;
