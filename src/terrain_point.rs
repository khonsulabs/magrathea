use crate::types::Kilometers;
use euclid::{Length, Point2D};

pub struct TerrainPoint {
    pub location: TerrainLocation,
    pub elevation: Length<f32, Kilometers>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct TerrainLocation {
    pub point: Point2D<f32, Kilometers>,
}

impl TerrainLocation {
    pub fn new(point: Point2D<f32, Kilometers>) -> Self {
        Self { point }
    }
}

impl spade::HasPosition for TerrainPoint {
    type Point = TerrainLocation;

    fn position(&self) -> Self::Point {
        self.location
    }
}

impl spade::PointN for TerrainLocation {
    type Scalar = f32;

    fn dimensions() -> usize {
        2
    }

    fn from_value(value: Self::Scalar) -> Self {
        Self {
            point: Point2D::new(value, value),
        }
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.point.x,
            1 => &self.point.y,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.point.x,
            1 => &mut self.point.y,
            _ => unreachable!(),
        }
    }
}
