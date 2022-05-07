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

#[derive(Debug)]
pub enum BvhNode {
    Leaf {
        aabb: AABB,
        index: usize,
    },
    Node {
        aabb: AABB,
        left: usize,
        right: usize,
    },
}

impl BvhNode {
    pub fn aabb(&self) -> AABB {
        match self {
            Self::Leaf { aabb, .. } => *aabb,
            Self::Node { aabb, .. } => *aabb,
        }
    }
}

// TODO: x, y, z_ord bool vec.
pub struct BvhTree {
    n_leafs: usize,
    nodes: Vec<BvhNode>,
    root: usize,
}

impl BvhTree {
    pub fn build_sweep<I: Iterator<Item = (usize, AABB)>>(iter: I) -> Self {
        let mut nodes: Vec<BvhNode> = iter
            .map(|(i, aabb)| BvhNode::Leaf { aabb, index: i })
            .collect();
        let n_leafs = nodes.len();
        let aabb = nodes.iter().map(|n| n.aabb()).reduce(AABB::grow).unwrap();
        let mut children: Vec<usize> = (0..n_leafs).collect();
        let root = Self::sweep(&mut nodes, n_leafs, aabb, &mut children);
        Self {
            n_leafs,
            nodes,
            root,
        }
    }
    fn sweep(
        nodes: &mut Vec<BvhNode>,
        n_leafs: usize,
        p_aabb: AABB,
        children: &mut [usize],
    ) -> usize {
        let (split_axis, split_axis_size) = p_aabb.largest_axis_with_size();

        // Order the children along the longest axis.
        // TODO: Implementation with 3 sorted lists.
        // as described here: https://graphics.cg.uni-saarland.de/courses/cg1-2018/slides/Building_good_BVHs.pdf
        match split_axis {
            Axis::X => children.sort_by(|a, b| {
                nodes[*a].aabb().centroid()[0]
                    .partial_cmp(&nodes[*b].aabb().centroid()[0])
                    .unwrap()
            }),
            Axis::Y => children.sort_by(|a, b| {
                nodes[*a].aabb().centroid()[1]
                    .partial_cmp(&nodes[*b].aabb().centroid()[1])
                    .unwrap()
            }),
            Axis::Z => children.sort_by(|a, b| {
                nodes[*a].aabb().centroid()[2]
                    .partial_cmp(&nodes[*b].aabb().centroid()[2])
                    .unwrap()
            }),
        }

        if children.len() == 1 {
            children[0]
        } else if children.len() == 2 {
            nodes.push(BvhNode::Node {
                aabb: p_aabb,
                left: children[0],
                right: children[1],
            });
            println!("Parent: {:?}", p_aabb);
            println!("");
            nodes.len() - 1
        } else {
            let mut min_sah = std::f32::MAX;
            let mut min_sah_idx = 0;
            let mut min_sah_l_aabb = nodes[children[0]].aabb();
            let mut min_sah_r_aabb = AABB::default();
            let p_sa = p_aabb.surface_area();
            for i in 0..(children.len() - 1) {
                // The left aabb can be grown with the iteration
                let l_aabb = min_sah_l_aabb.grow(nodes[children[i]].aabb());
                let l_sa = l_aabb.surface_area();
                // The left aabb has to be generated for each iteration.
                // This should always at leas iterate over the last leaf node.
                let r_aabb = ((i + 1)..children.len())
                    .map(|i| nodes[children[i]].aabb())
                    .fold(nodes[children[i + 1]].aabb(), AABB::grow);
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
            let r_node_i = Self::sweep(nodes, n_leafs, min_sah_r_aabb, r_children);
            let l_node_i = Self::sweep(nodes, n_leafs, min_sah_l_aabb, l_children);
            nodes.push(BvhNode::Node {
                aabb: p_aabb,
                left: l_node_i,
                right: r_node_i,
            });
            nodes.len() - 1
        }
    }

    pub fn print_nodes(&self, index: usize, indent: usize) {
        let mut indent_string = String::new();
        for i in 0..indent {
            indent_string.push(' ');
        }
        match self.nodes[index] {
            BvhNode::Node { left, right, aabb } => {
                println!(
                    "{}[index: {}, aabb: ({:?}, {:?}), left: {}, right: {}]",
                    indent_string, index, aabb.min, aabb.max, left, right
                );
                self.print_nodes(left, indent + 1);
                self.print_nodes(right, indent + 1);
            }
            BvhNode::Leaf { aabb, index: tri } => {
                println!(
                    "{}[index: {}, aabb: ({:?}, {:?}), tri: {}]",
                    indent_string, index, aabb.min, aabb.max, tri
                );
            }
        }
    }
    pub fn print(&self) {
        self.print_nodes(self.root, 0);
        println!("");
    }

    pub fn print_pivot(&self, index: usize, pivot: usize, indent: usize) {
        let mut indent_string = String::new();
        for i in 0..indent {
            indent_string.push(' ');
        }

        match self.nodes[index] {
            BvhNode::Node { left, right, aabb } => {
                println!("{}[index: {}, pivot: {}]", indent_string, index, pivot);
                self.print_pivot(left, index, indent + 1);
                self.print_pivot(right, pivot, indent + 1);
            }
            BvhNode::Leaf { aabb, index: tri } => {
                println!(
                    "{}[index: {}, pivot: {}, tri: {}]",
                    indent_string, index, pivot, tri
                );
            }
        }
    }

    fn generate_flat_pivot(&self) -> FlatBvhTree {
        let mut dst = Vec::<FlatBvhNode>::with_capacity(self.nodes.len());
        self.add_flat(&mut dst, self.root, 0);
        FlatBvhTree { nodes: dst }
    }

    ///
    /// The pivot of a node is the parent of the first parent node of it, that is a left node of
    /// its parent. The pivot can than be used in combination with the right "pointer" to generate
    /// the miss index.
    /// TODO: Test if preorder would be better.
    /// TODO: The right most nodes have a miss pointing to the right child of root. This has to be
    /// fixed.
    ///
    pub fn add_flat(&self, dst: &mut Vec<FlatBvhNode>, index: usize, dst_pivot: u32) -> usize {
        let aabb = self.nodes[index].aabb();
        let min = [aabb.min[0], aabb.min[1], aabb.min[2], 0.];
        let max = [aabb.max[0], aabb.max[1], aabb.max[2], 0.];
        match self.nodes[index] {
            BvhNode::Node { left, right, .. } => {
                let dst_index = dst.len();
                dst.push(FlatBvhNode {
                    ty: FlatBvhNode::TY_NODE,
                    miss: dst_pivot,
                    right: 0,
                    min,
                    max,
                });
                self.add_flat(dst, left, dst_index as u32);
                dst[dst_index as usize].right = self.add_flat(dst, right, dst_pivot) as u32;
                dst_index
            }
            BvhNode::Leaf { index: ext_idx, .. } => {
                let dst_index = dst.len();
                dst.push(FlatBvhNode {
                    ty: FlatBvhNode::TY_LEAF,
                    miss: dst_pivot,
                    right: ext_idx as u32,
                    min,
                    max,
                });
                dst_index
            }
        }
    }

    pub fn generate_iterative(&self) -> FlatBvhTree {
        let mut dst = Vec::<FlatBvhNode>::with_capacity(self.nodes.len());
        self.add_flat(&mut dst, self.root, 0);
        let mut dst = FlatBvhTree { nodes: dst };
        dst.pivot_to_miss();
        dst
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FlatBvhNode {
    pub ty: u32,
    pub right: u32,
    pub miss: u32,
    pub min: [f32; 4],
    pub max: [f32; 4],
}
impl FlatBvhNode {
    pub const TY_NODE: u32 = 0x00;
    pub const TY_LEAF: u32 = 0x01;
}

#[derive(Debug)]
pub struct FlatBvhTree {
    pub nodes: Vec<FlatBvhNode>,
}

impl FlatBvhTree {
    pub fn build_sweep<Item: Into<IndexedAABB>, I: Iterator<Item = Item>>(iter: I) -> Self {
        let mut children: Vec<IndexedAABB> = iter.map(|x| x.into()).collect();
        let n_leafs = children.len();
        let aabb = children.iter().map(|c| c.aabb).fold(children[0].aabb, AABB::grow);
        let mut nodes: Vec<FlatBvhNode> = Vec::new();
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
    ///
    ///
    ///             0
    ///         /       \
    ///     1               2
    ///   /   \           /   \
    /// 3       4       5       6
    ///
    /// In this example the pivot relation is as follows:
    /// 0 is the pivot of 1 and 4,
    /// 1 is the pivot of 3,
    /// 2 is the pivot of 5.
    ///
    /// The miss pointer of any node is then just the right pointer of its pivot.
    ///
    fn sweep_pivot(
        dst: &mut Vec<FlatBvhNode>,
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
            dst.push(FlatBvhNode {
                ty: FlatBvhNode::TY_LEAF,
                right: children[0].index as u32,
                miss: pivot as u32,
                min: [p_aabb.min[0], p_aabb.min[1], p_aabb.min[2], 0.],
                max: [p_aabb.max[0], p_aabb.max[1], p_aabb.max[2], 0.],
            });
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
            dst.push(FlatBvhNode {
                ty: FlatBvhNode::TY_NODE,
                right: 0,
                miss: pivot as u32,
                min: [p_aabb.min[0], p_aabb.min[1], p_aabb.min[2], 0.],
                max: [p_aabb.max[0], p_aabb.max[1], p_aabb.max[2], 0.],
            });
            let l_node_i = Self::sweep_pivot(dst, min_sah_l_aabb, l_children, node_i);
            let r_node_i = Self::sweep_pivot(dst, min_sah_r_aabb, r_children, pivot);
            dst[node_i].right = r_node_i as u32;
            dst[node_i].miss = pivot as u32;
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
            if i as u32 >= self.nodes[0].right && self.nodes[i].miss == 0 {
                // The right most node's pivot would be the root node.
                // To invalidate their misses and indicate a miss of the whole tree they are set to
                // 0.
                // The root node (0) cannot be a miss.
                // TODO: check weather it would be better to set it to N+1
                self.nodes[i].miss = 0;
            } else {
                self.nodes[i].miss = self.nodes[self.nodes[i].miss as usize].right;
            }
        }
        self.nodes[0].miss = 0;
    }
    pub fn print_rec(&self, index: usize, indent_string: &mut String) {
        println!("{}index: {}, {:?}", indent_string, index, self.nodes[index]);
        if self.nodes[index].ty == FlatBvhNode::TY_NODE {
            indent_string.push(' ');
            print!("l:");
            self.print_rec(index + 1, indent_string);
            print!("r:");
            self.print_rec(self.nodes[index].right as usize, indent_string);
            indent_string.pop();
        }
    }
}
