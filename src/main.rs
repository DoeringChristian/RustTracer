use std::ops::Range;


pub trait Pos3{
    fn pos3(&self) -> [f32; 3];
}

#[derive(Copy, Clone)]
pub struct Vert{
    pub pos: [f32; 4],
}

impl Pos3 for Vert{
    fn pos3(&self) -> [f32; 3] {
        [self.pos[0], self.pos[1], self.pos[2]]
    }
}

pub struct Mesh{
    pub verts: Vec<Vert>,
    pub tris: Vec<[usize; 3]>,
}

impl Mesh{
    pub fn get_tri(&self, index: usize) -> [Vert; 3]{
        [
            self.verts[self.tris[index][0]],
            self.verts[self.tris[index][1]],
            self.verts[self.tris[index][2]],
        ]
    }
    pub fn get_for_tri(&self, indices: &[usize; 3]) -> [Vert; 3]{
        [
            self.verts[indices[0]],
            self.verts[indices[1]],
            self.verts[indices[2]]
        ]
    }
}

pub enum Axis{
    X,
    Y,
    Z,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct AABB{
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB{
    pub fn grow(self, other: AABB) -> AABB{
        AABB{
            min: [self.min[0].min(other.min[0]), self.min[1].min(other.min[1]), self.min[2].min(other.min[2])],
            max: [self.max[0].max(other.max[0]), self.max[1].max(other.max[1]), self.max[2].max(other.max[2])],
        }
    }
    pub fn largest_axis(&self) -> Axis{
        self.largest_axis_with_size().0
    }
    pub fn largest_axis_with_size(&self) -> (Axis, f32){
        let x_size = self.max[0] - self.min[0];
        let y_size = self.max[1] - self.min[1];
        let z_size = self.max[2] - self.min[2];
        if x_size > y_size && x_size > z_size{
            (Axis::X, x_size)
        }
        else if y_size > x_size && y_size > z_size{
            (Axis::Y, y_size)
        }
        else{
            (Axis::Z, z_size)
        }
    }
    pub fn centroid(&self) -> [f32; 3]{
        [self.max[0]/2. + self.min[0]/2., self.max[1]/2. + self.min[1]/2., self.max[2]/2. + self.min[2]/2.]
    }
    /// Surface area of the AABB.
    pub fn surface_area(&self) -> f32{
        2. * (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
            + 2. * (self.max[1] - self.min[1]) * (self.max[2] - self.min[2])
            + 2. * (self.max[0] - self.min[0]) * (self.max[2] - self.min[2])
    }
}

impl From<[Vert; 3]> for AABB{
    fn from(src: [Vert; 3]) -> Self {
        let v1 = src[0].pos3();
        let v2 = src[1].pos3();
        let v3 = src[2].pos3();
        AABB{
            min: [v1[0].min(v2[0]).min(v3[0]), v1[1].min(v2[1]).min(v3[1]), v1[2].min(v2[2]).min(v3[2])],
            max: [v1[0].max(v2[0]).max(v3[0]), v1[1].max(v2[1]).max(v3[1]), v1[2].max(v2[2]).max(v3[2])],
        }
    }
}

#[derive(Debug)]
pub enum BvhNode{
    Leaf{
        aabb: AABB,
        index: usize,
    },
    Node{
        aabb: AABB,
        left: usize,
        right: usize,
    }
}

impl BvhNode{
    pub fn aabb(&self) -> AABB{
        match self{
            Self::Leaf{aabb, ..} => *aabb,
            Self::Node{aabb, ..} => *aabb,
        }
    }
}

// TODO: x, y, z_ord bool vec.
pub struct BvhTree{
    n_leafs: usize,
    nodes: Vec<BvhNode>,
    root: usize,
}

impl BvhTree{
    pub fn build_sweep(mesh: &Mesh) -> Self{
        let mut nodes: Vec<BvhNode> = mesh.tris.iter().enumerate().map(|(i, tri)|{
            BvhNode::Leaf{
                aabb: mesh.get_for_tri(tri).into(),
                index: i,
            }
        }).collect();
        let n_leafs = nodes.len();
        let aabb = nodes.iter().map(|n| n.aabb()).reduce(AABB::grow).unwrap();
        let mut children: Vec<usize> = (0..n_leafs).collect();
        let root = Self::sweep(&mut nodes, n_leafs, aabb, &mut children);
        Self{
            n_leafs,
            nodes,
            root,
        }
    }
    fn sweep(nodes: &mut Vec<BvhNode>, n_leafs: usize, p_aabb: AABB, children: &mut [usize]) -> usize{
        let (split_axis, split_axis_size) = p_aabb.largest_axis_with_size();

        // Order the children along the longest axis.
        // TODO: Implementation with 3 sorted lists.
        // as described here: https://graphics.cg.uni-saarland.de/courses/cg1-2018/slides/Building_good_BVHs.pdf
        match split_axis{
            Axis::X => children.sort_by(|a, b|{
                nodes[*a].aabb().centroid()[0].partial_cmp(&nodes[*b].aabb().centroid()[0]).unwrap()
            }),
            Axis::Y => children.sort_by(|a, b|{
                nodes[*a].aabb().centroid()[1].partial_cmp(&nodes[*b].aabb().centroid()[1]).unwrap()
            }),
            Axis::Z => children.sort_by(|a, b|{
                nodes[*a].aabb().centroid()[2].partial_cmp(&nodes[*b].aabb().centroid()[2]).unwrap()
            }),
        }

        if children.len() == 1{
            children[0]
        }
        else if children.len() == 2{
            nodes.push(BvhNode::Node{
                aabb: p_aabb,
                left: children[0],
                right: children[1],
            });
            println!("Parent: {:?}", p_aabb);
            println!("");
            nodes.len()-1
        }
        else{
            let mut min_sah = std::f32::MAX;
            let mut min_sah_idx = 0;
            let mut min_sah_l_aabb = nodes[children[0]].aabb();
            let mut min_sah_r_aabb = AABB::default();
            let p_sa = p_aabb.surface_area();
            for i in 0..(children.len()-1){
                // The left aabb can be grown with the iteration
                let l_aabb = min_sah_l_aabb.grow(nodes[children[i]].aabb());
                let l_sa = l_aabb.surface_area();
                // The left aabb has to be generated for each iteration.
                // This should always at leas iterate over the last leaf node.
                let r_aabb = ((i + 1)..children.len()).map(|i|{nodes[children[i]].aabb()}).fold(nodes[children[i+1]].aabb(), AABB::grow);
                let r_sa = r_aabb.surface_area();

                let sah = (l_sa + r_sa) / p_sa;
                if sah < min_sah{
                    min_sah = sah;
                    min_sah_idx = i;
                    min_sah_l_aabb = l_aabb;
                    min_sah_r_aabb = r_aabb;
                }
            }
            let (l_children, r_children) = children.split_at_mut(min_sah_idx+1);
            let r_node_i = Self::sweep(nodes, n_leafs, min_sah_r_aabb, r_children);
            let l_node_i = Self::sweep(nodes, n_leafs, min_sah_l_aabb, l_children);
            nodes.push(BvhNode::Node{
                aabb: p_aabb,
                left: l_node_i,
                right: r_node_i,
            });
            nodes.len()-1
        }
    }

    pub fn print_nodes(&self, index: usize, indent: usize){
        let mut indent_string = String::new();
        for i in 0..indent{
            indent_string.push(' ');
        }
        match self.nodes[index]{
            BvhNode::Node{left, right, aabb} => {
                println!("{}[index: {}, aabb: ({:?}, {:?}), left: {}, right: {}]", indent_string, index, aabb.min, aabb.max, left, right);
                self.print_nodes(left, indent + 1);
                self.print_nodes(right, indent + 1);
            },
            BvhNode::Leaf{aabb, index: tri} => {
                println!("{}[index: {}, aabb: ({:?}, {:?}), tri: {}]", indent_string, index, aabb.min, aabb.max, tri);
            }
        }
    }
    pub fn print(&self){
        self.print_nodes(self.root, 0);
        println!("");
    }

    pub fn print_pivot(&self, index: usize, pivot: usize, indent: usize){
        let mut indent_string = String::new();
        for i in 0..indent{
            indent_string.push(' ');
        }

        match self.nodes[index]{
            BvhNode::Node{left, right, aabb} => {
                println!("{}[index: {}, pivot: {}]", indent_string, index, pivot);
                self.print_pivot(left, index, indent+1);
                self.print_pivot(right, pivot, indent+1);
            },
            BvhNode::Leaf{aabb, index: tri} => {
                println!("{}[index: {}, pivot: {}, tri: {}]", indent_string, index, pivot, tri);
            }
        }
    }

    pub fn generate_flat_pivot(&self) -> FlatBvhTree{
        let mut dst = Vec::<FlatBvhNode>::with_capacity(self.nodes.len());
        self.add_flat(&mut dst, self.root, 0);
        FlatBvhTree{nodes: dst}
    }

    ///
    /// The pivot of a node is the parent of the first parent node of it, that is a left node of
    /// its parent. The pivot can than be used in combination with the right "pointer" to generate
    /// the miss index.
    /// TODO: Test if preorder would be better.
    /// TODO: The right most nodes have a miss pointing to the right child of root. This has to be
    /// fixed.
    ///
    pub fn add_flat(&self, dst: &mut Vec<FlatBvhNode>, index: usize, dst_pivot: u32) -> usize{
        let aabb = self.nodes[index].aabb();
        let min = [aabb.min[0], aabb.min[1], aabb.min[2], 0.];
        let max = [aabb.max[0], aabb.max[1], aabb.max[2], 0.];
        match self.nodes[index]{
            BvhNode::Node{left, right, ..} => {
                let dst_index = dst.len();
                dst.push(FlatBvhNode{
                    ty: FlatBvhNode::TY_NODE,
                    miss: dst_pivot,
                    right: 0,
                    min,
                    max,
                });
                self.add_flat(dst, left, dst_index as u32);
                dst[dst_index as usize].right = self.add_flat(dst, right, dst_pivot) as u32;
                dst_index
            },
            BvhNode::Leaf{index: ext_idx, ..} => {
                let dst_index = dst.len();
                dst.push(FlatBvhNode{
                    ty: FlatBvhNode::TY_LEAF,
                    miss: dst_pivot,
                    right: ext_idx as u32,
                    min, max,
                });
                dst_index
            }
        }
    }

    pub fn generate_iterative(&self) -> FlatBvhTree{
        let mut dst = Vec::<FlatBvhNode>::with_capacity(self.nodes.len());
        self.add_flat(&mut dst, self.root, 0);
        let mut dst = FlatBvhTree{nodes: dst};
        dst.pivot_to_direct();
        dst
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FlatBvhNode{
    pub ty: u32,
    pub right: u32,
    pub miss: u32,
    pub max: [f32; 4],
    pub min: [f32; 4],
}
impl FlatBvhNode{
    pub const TY_NODE: u32 = 0x00;
    pub const TY_LEAF: u32 = 0x01;
}

#[derive(Debug)]
pub struct FlatBvhTree{
    nodes: Vec<FlatBvhNode>,
}

impl FlatBvhTree{
    fn pivot_to_direct(&mut self){
        for i in 0..self.nodes.len(){
            if i as u32 >= self.nodes[0].right && self.nodes[i].miss == 0{
                // The right most node's pivot would be the root node.
                // To invalidate their misses and indicate a miss of the whole tree they are set to
                // 0.
                // The root node (0) cannot be a miss.
                self.nodes[i].miss = 0;
            }
            else{
                self.nodes[i].miss = self.nodes[self.nodes[i].miss as usize].right;
            }
        }
        self.nodes[0].miss = 0;
    }
}

fn main() {
    let verts = vec![
        Vert{pos: [0., 0., 0., 0.]},
        Vert{pos: [1., 0., 0., 0.]},
        Vert{pos: [1., 1., 0., 0.]},
        Vert{pos: [2., 0., 0., 0.]},
        Vert{pos: [3., 0., 0., 0.]},
        Vert{pos: [3., 1., 0., 0.]},
        Vert{pos: [2., 1., 0., 0.]},
        Vert{pos: [3., 1., 0., 0.]},
        Vert{pos: [3., 2., 0., 0.]},
    ];
    let tris = vec![
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
    ];

        let mesh = Mesh{
            verts,
            tris,
        };

        let bvh = BvhTree::build_sweep(&mesh);
        bvh.print();
        bvh.print_pivot(bvh.root, bvh.root, 0);
        let iterative = bvh.generate_iterative();
        for node in iterative.nodes{
            println!("{:?}", node);
        }
}
