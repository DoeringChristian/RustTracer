use std::marker::PhantomData;

use crate::{bvh::*, aabb::*};

pub type BVHBucketBuidler16<Item: Into<IndexedAABB>, Iter: Iterator<Item = Item>, Node: BVHNode> = BVHBucketBuidler<16, Item, Iter, Node>;

pub struct BVHBucketBuidler<const N: usize, Item: Into<IndexedAABB>, Iter: Iterator<Item = Item>, Node: BVHNode>{
    _item: PhantomData<Item>,
    _iter: PhantomData<Iter>,
    children: Vec<IndexedAABB>,
    aabb: AABB,
    nodes: Vec<Node>,
    buckets: Vec<Vec<IndexedAABB>>,
}

impl<const N: usize, Item: Into<IndexedAABB>, Iter: Iterator<Item = Item>, Node: BVHNode> BVHBucketBuidler<N, Item, Iter, Node>{
    pub fn from_iter(iter: Iter) -> Self{
        let children: Vec<IndexedAABB> = iter.map(|x| x.into()).collect();
        Self{
            _iter: PhantomData,
            _item: PhantomData,
            nodes: Vec::with_capacity(children.len() * 2),
            buckets: vec![Vec::with_capacity(children.len() / N); N],
            aabb: children.iter().map(|c| c.aabb).fold(AABB::empty(), AABB::grow),
            children,
        }
    }
    pub fn build(self) -> BVH<Node>{
        let mut nodes = self.nodes;
        let mut children = self.children;
        let mut buckets = self.buckets;
        let aabb = self.aabb;
        Self::buckets_pivot(&mut nodes, aabb, &mut children, &mut buckets, 0);
        let mut tree = BVH{nodes};
        tree.pivot_to_miss();
        tree
    }
    ///
    /// Same as sweep_pivot but with Buckets to speed up construction.
    ///
    fn buckets_pivot(
        dst: &mut Vec<Node>,
        p_aabb: AABB,
        children: &mut [IndexedAABB],
        buckets: &mut Vec<Vec<IndexedAABB>>,
        pivot: usize,
    ) -> usize {

        if children.len() == 1 {
            dst.push(Node::new_leaf(
                    p_aabb,
                    children[0].index,
                    pivot
            ));
            dst.len() - 1
        } else if children.len() == 2{
            let l_aabb = children[0].aabb;
            let r_aabb = children[1].aabb;

            let (l_children, r_children) = children.split_at_mut(1);
            let node_i = dst.len();
            dst.push(Node::new_node(
                    p_aabb,
                    0,
                    pivot,
            ));
            let l_node_i = Self::buckets_pivot(dst, l_aabb, l_children, buckets, node_i);
            let r_node_i = Self::buckets_pivot(dst, r_aabb, r_children, buckets, pivot);
            dst[node_i].set_right(r_node_i);
            //dst[node_i].right = r_node_i as u32;
            //dst[node_i].miss = pivot as u32;
            node_i
        } else {
            // clear all buckets.
            for bucket in buckets.iter_mut(){
                bucket.clear();
            }
            let centoid_aabb: AABB = children.iter().map(|c| c.aabb.centroid().into()).fold(AABB::empty(), AABB::grow);

            let (axis, split_axis_size) = centoid_aabb.largest_axis_with_size();
            let axis: usize = axis.into();

            let mut bucket_aabbs = [AABB::empty(); N];

            // Push the children into their respective buckets.
            for child in children.iter(){
                // The bucket number to which to push the child.
                // a      c        b
                // [   |   |   |   ]
                // n = ceil((c-a)/(b-a) * N) -1
                // TODO: safeguard for division.
                let n = (((child.aabb.centroid()[axis] - centoid_aabb.min[axis])/split_axis_size * (N as f32)).ceil()-1.) as usize;
                // Insert child into bucket.
                buckets[n].push(*child);
                // Grow the aabb corresponding to that bucket.
                bucket_aabbs[n] = bucket_aabbs[n].grow(child.aabb);
            }

            // Accumulate the bounding boxes of the buffers for the left and right side. This gives
            // linear speed.
            let mut l_bucket_aabb_acc = [AABB::empty(); N];
            let mut r_bucket_aabb_acc = [AABB::empty(); N];
            let mut l_aabb = AABB::empty();
            let mut r_aabb = AABB::empty();
            for i in 0..(N-1){
                l_aabb = l_aabb.grow(bucket_aabbs[i]);
                r_aabb = r_aabb.grow(bucket_aabbs[N-i-1]);
                l_bucket_aabb_acc[i] = l_aabb;
                r_bucket_aabb_acc[i] = r_aabb;
            }

            let mut min_sah = std::f32::INFINITY;
            let mut bucket_split = 0;
            let mut count_non_empty = 0;
            let p_sa = p_aabb.surface_area();
            for i in 0..(N-1){
                if !(buckets[i].is_empty()){
                    let l_sa = l_bucket_aabb_acc[i].surface_area();
                    let r_sa = r_bucket_aabb_acc[i].surface_area();
                    let sah = (l_sa + r_sa) / p_sa;
                    if sah < min_sah {
                        min_sah = sah;
                        bucket_split = i;
                    }
                    count_non_empty += 1;
                }
            }

            // Extract the aabbs of the left and right children.
            let l_abb = l_bucket_aabb_acc[bucket_split];
            let r_abb = r_bucket_aabb_acc[bucket_split];

            // Fill children back from bucket into children slice.
            let mut child_index = 0;
            let mut children_split = 0;
            for i in 0..N{
                for child in buckets[i].iter(){
                    children[child_index] = *child;
                    child_index += 1;
                }
                // When we have filled the children of the left buckets we keep the child_index;
                if i == bucket_split{
                    children_split = child_index;
                }
            }

            // If there should only be one bucket occupied (which could happen if all centroids are
            // in the sampe place) we just split them in 2.
            if count_non_empty == 1{
                children_split = children.len()/2;
            }


            // Split the children at the children_split index.
            let (l_children, r_children) = children.split_at_mut(children_split);
            let node_i = dst.len();
            dst.push(Node::new_node(
                    p_aabb,
                    0,
                    pivot,
            ));
            let l_node_i = Self::buckets_pivot(dst, l_abb, l_children, buckets, node_i);
            let r_node_i = Self::buckets_pivot(dst, r_abb, r_children, buckets, pivot);
            dst[node_i].set_right(r_node_i);
            //dst[node_i].right = r_node_i as u32;
            //dst[node_i].miss = pivot as u32;
            node_i
        }
    }
}
