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
            min: [v1[0].min(v2[0]).min(v3[0]), v1[1].min(v2[1]).min(v3[1]), v1[3].min(v2[3]).min(v3[3])],
            max: [v1[0].max(v2[0]).max(v3[0]), v1[1].max(v2[1]).max(v3[1]), v1[3].max(v2[3]).max(v3[3])],
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
#[derive(Debug)]
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
            nodes.len()-1
        }
        else{
            let mut min_sa = std::f32::MAX;
            let mut min_sa_idx = 0;
            let mut r_aabb = AABB::default();
            let mut l_aabb = AABB::default();
            for i in 0..(children.len()-1){
                // The right aabb can be grown with the iteration
                r_aabb = r_aabb.grow(nodes[children[i]].aabb());
                let r_sa = r_aabb.surface_area();
                // The left aabb has to be generated for each iteration.
                // This should always at leas iterate over the last leaf node.
                l_aabb = ((i + 1)..children.len()).map(|i|{nodes[children[i]].aabb()}).reduce(AABB::grow).unwrap();
                let l_sa = l_aabb.surface_area();

                let sa = r_sa + l_sa;
                if sa < min_sa{
                    min_sa = sa;
                    min_sa_idx = i;
                }
            }
            let (l_children, r_children) = children.split_at_mut(min_sa_idx+1);
            let l_node_i = Self::sweep(nodes, n_leafs, l_aabb, l_children);
            let r_node_i = Self::sweep(nodes, n_leafs, r_aabb, r_children);
            nodes.push(BvhNode::Node{
                aabb: p_aabb,
                left: l_node_i,
                right: r_node_i,
            });
            nodes.len()-1
        }
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
    ];
    let tris = vec![
        [0, 1, 2],
        [3, 4, 5],
    ];

    let mesh = Mesh{
        verts,
        tris,
    };

    let bvh = BvhTree::build_sweep(&mesh);
    println!("{:?}", bvh);
}
