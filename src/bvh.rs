use crate::aabb::*;

pub trait BVHNode {
    type ExternIndex: Copy + Clone + std::fmt::Debug;
    fn new_node(aabb: AABB, right: usize, miss: usize) -> Self;
    fn new_leaf(aabb: AABB, index: Self::ExternIndex, miss: usize) -> Self;
    fn set_right(&mut self, right: usize);
    fn set_miss(&mut self, miss: usize);
    fn right(&self) -> usize;
    fn miss(&self) -> usize;
    fn is_leaf(&self) -> bool;
    fn is_node(&self) -> bool;
}

///
/// TODO: Implement Bucket methode.
///
#[derive(Debug)]
pub struct BVH<Node: BVHNode> {
    pub nodes: Vec<Node>,
    aabb: AABB,
}

impl<Node: BVHNode> BVH<Node> {
    pub fn build_sweep<Item: Into<IndexedAABB<Node::ExternIndex>>, I: Iterator<Item = Item>>(
        iter: I,
    ) -> Self {
        let mut children: Vec<IndexedAABB<Node::ExternIndex>> = iter.map(|x| x.into()).collect();
        let aabb = children
            .iter()
            .map(|c| c.aabb)
            .fold(children[0].aabb, AABB::grow);
        let mut nodes: Vec<Node> = Vec::new();
        Self::sweep_pivot(&mut nodes, aabb, &mut children, 0);
        let mut tree = Self { nodes, aabb};
        Self::pivot_to_miss(&mut tree);
        tree
    }
    ///
    /// Generates the BVH into the dst vector with the `miss` parameter being the pivot of that
    /// node.
    /// * `dst` vector into which the BVH is generate.
    /// * `p_aabb` the aabb of the parent.
    /// * `children` the children of the parent who are split into two parts.
    /// * `pivot` the pivot of the parent node. The pivot of a node is the parent of the first
    /// parent, that is a left node.
    /// It can also be thought of as the lowest common ancestor of the tree.
    ///
    ///```text
    ///             0
    ///         /       \
    ///     1               2
    ///   /   \           /   \
    /// 3       4       5       6
    ///
    ///```
    ///
    /// In this example the pivot relation is as follows:
    /// 0 is the pivot of 1 and 4,
    /// 1 is the pivot of 3,
    /// 2 is the pivot of 5.
    ///
    /// For the pivot following rules hold:
    /// If N is a node in the tree, N_L, N_R are its left/right children.
    /// p(N) is the pivot of N for any node N.
    /// Then p(N_L) = N and p(N_R) = p(N)
    ///
    ///
    /// The miss pointer of any node is then just the right pointer of its pivot.
    ///
    fn sweep_pivot(
        dst: &mut Vec<Node>,
        p_aabb: AABB,
        children: &mut [IndexedAABB<Node::ExternIndex>],
        pivot: usize,
    ) -> usize {
        let (split_axis, split_axis_size) = p_aabb.largest_axis_with_size();

        // Order the children along the longest axis.
        // TODO: Implementation with 3 sorted lists.
        // as described here: https://graphics.cg.uni-saarland.de/courses/cg1-2018/slides/Building_good_BVHs.pdf
        match split_axis {
            Axis::X => children.sort_by(|a, b| {
                a.aabb.centroid()[0]
                    .partial_cmp(&b.aabb.centroid()[0])
                    .unwrap()
            }),
            Axis::Y => children.sort_by(|a, b| {
                a.aabb.centroid()[1]
                    .partial_cmp(&b.aabb.centroid()[1])
                    .unwrap()
            }),
            Axis::Z => children.sort_by(|a, b| {
                a.aabb.centroid()[2]
                    .partial_cmp(&b.aabb.centroid()[2])
                    .unwrap()
            }),
        }

        if children.len() == 1 {
            dst.push(Node::new_leaf(p_aabb, children[0].index, pivot));
            dst.len() - 1
        } else {
            let mut min_sah = std::f32::MAX;
            let mut min_sah_idx = 0;
            let mut min_sah_l_aabb = children[0].aabb;
            let mut min_sah_r_aabb = AABB::default();
            let p_sa = p_aabb.surface_area();

            for i in 0..(children.len() - 1) {
                // The left aabb can be grown with the iteration
                let l_aabb = min_sah_l_aabb.grow(children[i].aabb);
                let l_sa = l_aabb.surface_area();

                // The right aabb has to be generated for each iteration.
                // This should always at least iterate over the last leaf node.
                let r_aabb = ((i + 1)..children.len())
                    .map(|i| children[i].aabb)
                    .fold(children[i + 1].aabb, AABB::grow);
                let r_sa = r_aabb.surface_area();

                let sah = (l_sa + r_sa) / p_sa;
                if sah < min_sah {
                    min_sah = sah;
                    min_sah_idx = i;
                    min_sah_l_aabb = l_aabb;
                    min_sah_r_aabb = r_aabb;
                }
            }
            let (l_children, r_children) = children.split_at_mut(min_sah_idx + 1);
            let node_i = dst.len();
            dst.push(Node::new_node(p_aabb, 0, pivot));
            let _l_node_i = Self::sweep_pivot(dst, min_sah_l_aabb, l_children, node_i);
            let r_node_i = Self::sweep_pivot(dst, min_sah_r_aabb, r_children, pivot);
            dst[node_i].set_right(r_node_i);
            //dst[node_i].right = r_node_i as u32;
            //dst[node_i].miss = pivot as u32;
            node_i
        }
    }
    pub fn build_buckets_8<Item: Into<IndexedAABB<Node::ExternIndex>>, I: Iterator<Item = Item>>(
        iter: I,
    ) -> Self {
        Self::build_buckets_num::<8, Item, I>(iter)
    }
    pub fn build_buckets_16<
        Item: Into<IndexedAABB<Node::ExternIndex>>,
        I: Iterator<Item = Item>,
    >(
        iter: I,
    ) -> Self {
        Self::build_buckets_num::<16, Item, I>(iter)
    }
    pub fn build_buckets_num<
        const N: usize,
        Item: Into<IndexedAABB<Node::ExternIndex>>,
        I: Iterator<Item = Item>,
    >(
        iter: I,
    ) -> Self {
        let mut children: Vec<IndexedAABB<Node::ExternIndex>> = iter.map(|x| x.into()).collect();
        let aabb = children
            .iter()
            .map(|c| c.aabb)
            .fold(children[0].aabb, AABB::grow);
        let mut nodes: Vec<Node> = Vec::new();
        let mut buckets = vec![Vec::new(); N];
        Self::buckets_pivot::<N>(&mut nodes, aabb, &mut children, &mut buckets, 0);
        let mut tree = Self { nodes, aabb };
        Self::pivot_to_miss(&mut tree);
        tree
    }
    ///
    /// Same as sweep_pivot but with Buckets to speed up construction.
    /// Generates the BVH into the dst vector with the `miss` parameter being the pivot of that
    /// node.
    /// * `dst` vector into which the BVH is generate.
    /// * `p_aabb` the aabb of the parent.
    /// * `children` the children of the parent who are split into two parts.
    /// * `pivot` the pivot of the parent node. The pivot of a node is the parent of the first
    /// parent, that is a left node.
    /// It can also be thought of as the lowest common ancestor of the tree.
    ///
    ///```text
    ///             0
    ///         /       \
    ///     1               2
    ///   /   \           /   \
    /// 3       4       5       6
    ///
    ///```
    ///
    /// In this example the pivot relation is as follows:
    /// 0 is the pivot of 1 and 4,
    /// 1 is the pivot of 3,
    /// 2 is the pivot of 5.
    ///
    /// For the pivot following rules hold:
    /// If N is a node in the tree, N_L, N_R are its left/right children.
    /// p(N) is the pivot of N for any node N.
    /// Then p(N_L) = N and p(N_R) = p(N)
    ///
    ///
    /// The miss pointer of any node is then just the right pointer of its pivot.
    ///
    fn buckets_pivot<const N: usize>(
        dst: &mut Vec<Node>,
        p_aabb: AABB,
        children: &mut [IndexedAABB<Node::ExternIndex>],
        buckets: &mut Vec<Vec<IndexedAABB<Node::ExternIndex>>>,
        pivot: usize,
    ) -> usize {
        if children.len() == 1 {
            dst.push(Node::new_leaf(p_aabb, children[0].index, pivot));
            dst.len() - 1
        } else if children.len() == 2 {
            // Fast track if there are only two children left.
            //
            let l_aabb = children[0].aabb;
            let r_aabb = children[1].aabb;

            let (l_children, r_children) = children.split_at_mut(1);
            let node_i = dst.len();
            dst.push(Node::new_node(p_aabb, 0, pivot));
            let _l_node_i = Self::buckets_pivot::<N>(dst, l_aabb, l_children, buckets, node_i);
            let r_node_i = Self::buckets_pivot::<N>(dst, r_aabb, r_children, buckets, pivot);
            dst[node_i].set_right(r_node_i);
            node_i
        } else {
            // clear all buckets.
            // The buckets Vecs should not have any performance overhead since they probably
            // decreace in size only.
            for bucket in buckets.iter_mut() {
                bucket.clear();
            }
            let centoid_aabb: AABB = children
                .iter()
                .map(|c| c.aabb.centroid().into())
                .fold(AABB::empty(), AABB::grow);

            let (axis, split_axis_size) = centoid_aabb.largest_axis_with_size();
            let axis: usize = axis.into();

            let mut bucket_aabbs = [AABB::empty(); N];

            // Push the children into their respective buckets.
            for child in children.iter() {
                // The bucket number to which to push the child.
                // a      c        b
                // [   |   |   |   ]
                // n = ceil((c-a)/(b-a) * N) -1
                // TODO: safeguard for division.
                let n = (((child.aabb.centroid()[axis] - centoid_aabb.min[axis]) / split_axis_size
                    * (N as f32))
                    .ceil()
                    - 1.) as usize;
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
            for i in 0..(N - 1) {
                l_aabb = l_aabb.grow(bucket_aabbs[i]);
                r_aabb = r_aabb.grow(bucket_aabbs[N - i - 1]);
                l_bucket_aabb_acc[i] = l_aabb;
                r_bucket_aabb_acc[i] = r_aabb;
            }

            // Find the bucket after which we should split.
            let mut min_sah = std::f32::INFINITY;
            let mut bucket_split = 0;
            let mut count_non_empty = 0;
            let p_sa = p_aabb.surface_area();
            for i in 0..(N - 1) {
                if !(buckets[i].is_empty()) {
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
            for i in 0..N {
                for child in buckets[i].iter() {
                    children[child_index] = *child;
                    child_index += 1;
                }
                // When we have filled the children of the left buckets we keep the child_index;
                if i == bucket_split {
                    children_split = child_index;
                }
            }

            // If there should only be one bucket occupied (which could happen if all centroids are
            // in the sampe place) we just split them in 2.
            if count_non_empty == 1 {
                children_split = children.len() / 2;
            }

            // Split the children at the children_split index.
            let (l_children, r_children) = children.split_at_mut(children_split);
            let node_i = dst.len();
            dst.push(Node::new_node(p_aabb, 0, pivot));
            let _l_node_i = Self::buckets_pivot::<N>(dst, l_abb, l_children, buckets, node_i);
            let r_node_i = Self::buckets_pivot::<N>(dst, r_abb, r_children, buckets, pivot);
            dst[node_i].set_right(r_node_i);
            //dst[node_i].right = r_node_i as u32;
            //dst[node_i].miss = pivot as u32;
            node_i
        }
    }
    ///
    /// Change the pivot stored in the miss "pointer" to the miss pointer by setting it to the
    /// right node of the pivot.
    /// This needs to be done after generating the tree with the sweep_pivot methode.
    ///
    pub(crate) fn pivot_to_miss(&mut self) {
        for i in 0..self.nodes.len() {
            if i >= self.nodes[0].right() && self.nodes[i].miss() == 0 {
                // The right most node's pivot would be the root node.
                // To invalidate their misses and indicate a miss of the whole tree they are set to
                // 0.
                // The root node (0) cannot be a miss.
                // TODO: check weather it would be better to set it to N+1
                self.nodes[i].set_miss(0);
            } else {
                let miss = self.nodes[i].miss();
                let miss = self.nodes[miss].right();
                self.nodes[i].set_miss(miss);
            }
        }
        self.nodes[0].set_miss(0);
    }
    ///
    /// Returns AABB of this BVH. This can be used to generate a TLAS.
    ///
    pub fn aabb(&self) -> AABB{
        self.aabb
    }

    pub fn nodes(&self) -> &[Node]{
        &self.nodes
    }
}
impl<Node: BVHNode + std::fmt::Debug> BVH<Node> {
    pub fn print_rec(&self, index: usize, indent_string: &mut String) {
        println!("{}index: {}, {:?}", indent_string, index, self.nodes[index]);
        if self.nodes[index].is_node() {
            indent_string.push(' ');
            print!("l:");
            self.print_rec(index + 1, indent_string);
            print!("r:");
            self.print_rec(self.nodes[index].right(), indent_string);
            indent_string.pop();
        }
    }
}
#[cfg(test)]
mod test {
    use crate::aabb::*;
    use crate::bvh::*;
    use crate::glsl_bvh::*;

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

    fn load_suzanne() -> Mesh {
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
            })
            .collect();

        let tris = (0..(suzanne[0].mesh.indices.len() / 3))
            .into_iter()
            .map(|i| {
                [
                    suzanne[0].mesh.indices[i * 3] as usize,
                    suzanne[0].mesh.indices[i * 3 + 1] as usize,
                    suzanne[0].mesh.indices[i * 3 + 2] as usize,
                ]
            })
            .collect();

        let mesh = Mesh { verts, tris };
        mesh
    }

    #[test]
    pub fn test_suzanne_8() {
        let mesh = load_suzanne();

        {
            time_test::time_test!("build_buckets_8");
            for _ in 0..1 {
                GlslBVH::build_buckets_8(
                    mesh.tris
                        .iter()
                        .enumerate()
                        .map(|(i, tri)| (i, mesh.get_for_tri(tri).into())),
                );
            }
        }
    }

    #[test]
    pub fn test_suzanne_16() {
        let mesh = load_suzanne();

        {
            time_test::time_test!("build_buckets_16");
            for _ in 0..1 {
                GlslBVH::build_buckets_16(
                    mesh.tris
                        .iter()
                        .enumerate()
                        .map(|(i, tri)| (i, mesh.get_for_tri(tri).into())),
                );
            }
        }
    }
}
