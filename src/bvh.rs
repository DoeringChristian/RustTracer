
use crate::aabb::*;

pub trait BVHNode{
    fn new_node(aabb: AABB, right: usize, miss: usize) -> Self;
    fn new_leaf(aabb: AABB, index: usize, miss: usize) -> Self;
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
pub struct BVH<Node: BVHNode>{
    pub nodes: Vec<Node>,
}

impl<Node: BVHNode> BVH<Node> {
    pub fn build_sweep<Item: Into<IndexedAABB>, I: Iterator<Item = Item>>(iter: I) -> Self {
        let mut children: Vec<IndexedAABB> = iter.map(|x| x.into()).collect();
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
            //println!("{:?}", p_aabb);
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
impl<Node: BVHNode + std::fmt::Debug> BVH<Node>{
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
