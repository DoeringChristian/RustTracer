use std::iter::Enumerate;

pub enum Axis {
    X,
    Y,
    Z,
}

impl From<(usize, AABB)> for IndexedAABB{
    fn from(src: (usize, AABB)) -> Self {
        IndexedAABB{
            index: src.0,
            aabb: src.1,
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct IndexedAABB{
    pub index: usize,
    pub aabb: AABB,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB {
    pub fn grow(self, other: AABB) -> AABB {
        AABB {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }
    pub fn largest_axis(&self) -> Axis {
        self.largest_axis_with_size().0
    }
    pub fn largest_axis_with_size(&self) -> (Axis, f32) {
        let x_size = self.max[0] - self.min[0];
        let y_size = self.max[1] - self.min[1];
        let z_size = self.max[2] - self.min[2];
        if x_size > y_size && x_size > z_size {
            (Axis::X, x_size)
        } else if y_size > x_size && y_size > z_size {
            (Axis::Y, y_size)
        } else {
            (Axis::Z, z_size)
        }
    }
    pub fn centroid(&self) -> [f32; 3] {
        [
            self.max[0] / 2. + self.min[0] / 2.,
            self.max[1] / 2. + self.min[1] / 2.,
            self.max[2] / 2. + self.min[2] / 2.,
        ]
    }
    /// Surface area of the AABB.
    pub fn surface_area(&self) -> f32 {
        2. * (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
            + 2. * (self.max[1] - self.min[1]) * (self.max[2] - self.min[2])
            + 2. * (self.max[0] - self.min[0]) * (self.max[2] - self.min[2])
    }
}

pub trait BvhNode{
    fn new_node(aabb: AABB, right: usize, miss: usize) -> Self;
    fn new_leaf(aabb: AABB, index: usize, miss: usize) -> Self;
    fn set_right(&mut self, right: usize);
    fn set_miss(&mut self, miss: usize);
    fn right(&self) -> usize;
    fn miss(&self) -> usize;
    fn is_leaf(&self) -> bool;
    fn is_node(&self) -> bool;
}

#[repr(C)]
#[derive(Debug)]
pub struct FlatBvhNode {
    pub min: [f32; 4],
    pub max: [f32; 4],
    pub ty: u32,
    pub right: u32,
    pub miss: u32,
}
impl FlatBvhNode {
    pub const TY_NODE: u32 = 0x00;
    pub const TY_LEAF: u32 = 0x01;
}
impl BvhNode for FlatBvhNode{
    #[inline]
    fn new_node(aabb: AABB, right: usize, miss: usize) -> Self {
        FlatBvhNode{
            ty: Self::TY_NODE,
            min: [aabb.min[0], aabb.min[1], aabb.min[2], 0.],
            max: [aabb.max[0], aabb.max[1], aabb.max[2], 0.],
            right: right as u32,
            miss: miss as u32,
        }
    }

    #[inline]
    fn new_leaf(aabb: AABB, index: usize, miss: usize) -> Self {
        FlatBvhNode{
            ty: Self::TY_LEAF,
            min: [aabb.min[0], aabb.min[1], aabb.min[2], 0.],
            max: [aabb.max[0], aabb.max[1], aabb.max[2], 0.],
            right: index as u32,
            miss: miss as u32,
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

#[derive(Debug)]
pub struct BVH<Node: BvhNode>{
    pub nodes: Vec<Node>,
}

impl<Node: BvhNode> BVH<Node> {
    pub fn build_sweep<Item: Into<IndexedAABB>, I: Iterator<Item = Item>>(iter: I) -> Self {
        let mut children: Vec<IndexedAABB> = iter.map(|x| x.into()).collect();
        let n_leafs = children.len();
        let aabb = children.iter().map(|c| c.aabb).fold(children[0].aabb, AABB::grow);
        let mut nodes: Vec<Node> = Vec::new();
        Self::sweep_pivot(&mut nodes, aabb, &mut children, 0);
        let mut tree = Self{nodes};
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
        children: &mut [IndexedAABB],
        pivot: usize,
    ) -> usize {
        let (split_axis, split_axis_size) = p_aabb.largest_axis_with_size();

        // Order the children along the longest axis.
        // TODO: Implementation with 3 sorted lists.
        // as described here: https://graphics.cg.uni-saarland.de/courses/cg1-2018/slides/Building_good_BVHs.pdf
        match split_axis {
            Axis::X => {
                children.sort_by(|a, b| a.aabb.centroid()[0].partial_cmp(&b.aabb.centroid()[0]).unwrap())
            }
            Axis::Y => {
                children.sort_by(|a, b| a.aabb.centroid()[1].partial_cmp(&b.aabb.centroid()[1]).unwrap())
            }
            Axis::Z => {
                children.sort_by(|a, b| a.aabb.centroid()[2].partial_cmp(&b.aabb.centroid()[2]).unwrap())
            }
        }

        if children.len() == 1 {
            dst.push(Node::new_leaf(
                    p_aabb,
                    children[0].index,
                    pivot
            ));
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
            dst.push(Node::new_node(
                    p_aabb,
                    0,
                    pivot,
            ));
            let l_node_i = Self::sweep_pivot(dst, min_sah_l_aabb, l_children, node_i);
            let r_node_i = Self::sweep_pivot(dst, min_sah_r_aabb, r_children, pivot);
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
    fn pivot_to_miss(&mut self) {
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
}
impl<Node: BvhNode + std::fmt::Debug> BVH<Node>{
    pub fn print_rec(&self, index: usize, indent_string: &mut String) {
        println!("{}index: {}, {:?}", indent_string, index, self.nodes[index]);
        if self.nodes[index].is_node(){
            indent_string.push(' ');
            print!("l:");
            self.print_rec(index + 1, indent_string);
            print!("r:");
            self.print_rec(self.nodes[index].right(), indent_string);
            indent_string.pop();
        }
    }
}
