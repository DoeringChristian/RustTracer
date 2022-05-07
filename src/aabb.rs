
pub enum Axis {
    X,
    Y,
    Z,
}

impl From<Axis> for usize{
    #[inline]
    fn from(src: Axis) -> Self {
        match src{
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
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
    pub fn empty() -> Self{
        Self{
            min: [std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY],
            max: [std::f32::NEG_INFINITY, std::f32::NEG_INFINITY, std::f32::NEG_INFINITY],
        }
    }
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

impl From<[f32; 3]> for AABB{
    #[inline]
    fn from(src: [f32; 3]) -> Self {
        AABB{
            min: src,
            max: src,
        }
    }
}

