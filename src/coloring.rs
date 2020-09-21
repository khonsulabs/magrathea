use crate::elevation::Elevation;
use palette::Srgb;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Coloring {
    pub color: Srgb<f32>,
    pub elevation: Elevation,
}

impl PartialOrd for Coloring {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.elevation.partial_cmp(&other.elevation)
    }
}

impl spade::HasPosition for Coloring {
    type Point = Elevation;

    fn position(&self) -> Self::Point {
        self.elevation
    }
}
