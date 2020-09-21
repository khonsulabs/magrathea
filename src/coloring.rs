use crate::types::Kilometers;
use euclid::Length;
use palette::Srgb;
use sorted_vec::partial::SortedVec;

/// A pairing of an elevation and a color
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ElevationColor {
    /// The color used for this elevation
    pub color: Srgb<f32>,

    /// The elevation of this color
    pub elevation: Length<f32, Kilometers>,
}

impl PartialOrd for ElevationColor {
    /// partial_cmp only compares against elevation
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.elevation.partial_cmp(&other.elevation)
    }
}

impl ElevationColor {
    /// Constructor to take RGB byte components and an elevation
    pub fn from_u8(r: u8, g: u8, b: u8, elevation: Length<f32, Kilometers>) -> Self {
        Self {
            color: Srgb::new(r, g, b).into_format(),
            elevation,
        }
    }

    /// A basic elevation color palette that kinda resembles an earthlike planet
    pub fn earthlike() -> SortedVec<ElevationColor> {
        SortedVec::from_unsorted(vec![
            // Deep Ocean
            ElevationColor::from_u8(19, 30, 180, Kilometers::new(-1000.)),
            // Shallow Ocean
            ElevationColor::from_u8(98u8, 125, 223, Kilometers::new(0.)),
            // Beach
            ElevationColor::from_u8(209u8, 207, 169, Kilometers::new(100.)),
            // Grass
            ElevationColor::from_u8(152u8, 214, 102, Kilometers::new(200.)),
            // Forest
            ElevationColor::from_u8(47u8, 106, 42, Kilometers::new(600.)),
            // Mountain
            ElevationColor::from_u8(100u8, 73, 53, Kilometers::new(1600.)),
            // Snow
            ElevationColor::from_u8(238u8, 246, 245, Kilometers::new(1700.)),
        ])
    }

    /// A basic elevation color palette that kinda resembles a star like our sun
    pub fn sunlike() -> SortedVec<ElevationColor> {
        SortedVec::from_unsorted(vec![
            // Deep base glow
            ElevationColor::from_u8(189, 31, 10, Kilometers::new(0.)),
            // Bright middle
            ElevationColor::from_u8(250, 156, 56, Kilometers::new(20.)),
            // Hot top
            ElevationColor::from_u8(255, 218, 41, Kilometers::new(400.)),
        ])
    }
}
