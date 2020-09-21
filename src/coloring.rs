use crate::elevation::Elevation;
use palette::Srgb;
use sorted_vec::partial::SortedVec;

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

impl Coloring {
    pub fn from_u8(r: u8, g: u8, b: u8, elevation: f32) -> Self {
        Self {
            color: Srgb::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.),
            elevation: Elevation(elevation),
        }
    }

    pub fn earthlike() -> SortedVec<Coloring> {
        SortedVec::from_unsorted(vec![
            // Deep Ocean
            Coloring::from_u8(19, 30, 180, -1000.),
            // Shallow Ocean
            Coloring::from_u8(98u8, 125, 223, 0.),
            // Beach
            Coloring::from_u8(209u8, 207, 169, 100.),
            // Grass
            Coloring::from_u8(152u8, 214, 102, 200.),
            // Forest
            Coloring::from_u8(47u8, 106, 42, 600.),
            // Mountain
            // Coloring::from_u8(100u8, 73, 53, 1600.),
            // Snow
            // Coloring::from_u8(238u8, 246, 245, 1700.),
        ])
    }
}
