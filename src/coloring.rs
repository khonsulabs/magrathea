use crate::{planet::SurfaceDefinition, types::Kilometers};
use euclid::Length;
use palette::Srgb;

/// A pairing of an elevation and a color
#[derive(Clone, Copy, Debug)]
pub struct ElevationColor<Kind> {
    pub kind: Kind,

    /// The color used for this elevation
    pub color: Srgb<f32>,

    /// The elevation of this color
    pub elevation: Length<f32, Kilometers>,
}

impl<Kind> PartialOrd for ElevationColor<Kind> {
    /// partial_cmp only compares against elevation
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.elevation.partial_cmp(&other.elevation)
    }
}

impl<Kind> PartialEq for ElevationColor<Kind> {
    fn eq(&self, other: &Self) -> bool {
        self.elevation == other.elevation
    }
}

impl<Kind> ElevationColor<Kind> {
    /// Constructor to take RGB byte components and an elevation
    pub fn from_u8(kind: Kind, r: u8, g: u8, b: u8, elevation: Length<f32, Kilometers>) -> Self {
        Self {
            kind,
            color: Srgb::new(r, g, b).into_format(),
            elevation,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Earthlike {
    DeepOcean,
    ShallowOcean,
    Beach,
    Grass,
    Forest,
    Mountain,
    Snow,
}

impl ElevationColor<Earthlike> {
    /// A basic elevation color palette that kinda resembles an earthlike planet
    pub fn earthlike() -> Vec<Self> {
        vec![
            ElevationColor::from_u8(Earthlike::DeepOcean, 19, 30, 180, Kilometers::new(-2000.)),
            ElevationColor::from_u8(Earthlike::ShallowOcean, 98, 125, 223, Kilometers::new(0.)),
            ElevationColor::from_u8(Earthlike::Beach, 209, 207, 169, Kilometers::new(100.)),
            ElevationColor::from_u8(Earthlike::Grass, 152, 214, 102, Kilometers::new(200.)),
            ElevationColor::from_u8(Earthlike::Forest, 47, 106, 42, Kilometers::new(600.)),
            ElevationColor::from_u8(Earthlike::Mountain, 100, 73, 53, Kilometers::new(1600.)),
            ElevationColor::from_u8(Earthlike::Snow, 238, 246, 245, Kilometers::new(1700.)),
        ]
    }
}

impl SurfaceDefinition for Earthlike {
    fn max_chaos() -> f32 {
        7.
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Sunlike {
    DeepBase,
    BrightMiddle,
    HotTop,
}

impl ElevationColor<Sunlike> {
    /// A basic elevation color palette that kinda resembles a star like our sun
    pub fn sunlike() -> Vec<Self> {
        vec![
            // Deep base glow
            ElevationColor::from_u8(Sunlike::DeepBase, 189, 31, 10, Kilometers::new(-200.)),
            // Bright middle
            ElevationColor::from_u8(Sunlike::BrightMiddle, 250, 156, 56, Kilometers::new(-180.)),
            // Hot top
            ElevationColor::from_u8(Sunlike::HotTop, 255, 218, 41, Kilometers::new(200.)),
        ]
    }
}

impl SurfaceDefinition for Sunlike {
    fn max_chaos() -> f32 {
        20.
    }
}
