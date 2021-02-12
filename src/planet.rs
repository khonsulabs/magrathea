use std::{collections::HashMap, hash::Hash};

use crate::{coloring::ElevationColor, terrain::Terrain, types::Kilometers};
use euclid::{Angle, Length, Point2D, Rotation2D};
use palette::Srgb;
use sorted_vec::partial::SortedVec;
use uuid::Uuid;

/// A Procedural Planet definition
#[derive(Debug)]
pub struct Planet<Kind> {
    /// The unique value that is used to seed the random number generator
    pub seed: Uuid,

    /// The origin of the planet relative to the star it orbits
    pub origin: Point2D<f32, Kilometers>,

    /// The radius of the planet
    pub radius: Length<f32, Kilometers>,

    /// The ElevationColors used to generate the terrain
    pub colors: SortedVec<ElevationColor<Kind>>,
}

pub struct GeneratedPlanet<Kind> {
    pub image: image::RgbaImage,
    pub stats: HashMap<Kind, u32>,
}

impl<Kind> Planet<Kind>
where
    Kind: Clone + Hash + Eq,
{
    pub fn new_from_iter<I: IntoIterator<Item = ElevationColor<Kind>>>(seed: Uuid, origin: Point2D<f32, Kilometers>, radius: Length<f32, Kilometers>, colors: I) -> Self {
        Self {
            seed,
            origin,
            radius,
            colors: SortedVec::from_unsorted(colors.into_iter().collect()),
        }
    }
    /// Generates an image of `pixels` wide, and `pixels` tall. If a light is provided
    /// a shadow is simulated, and the colors are mixed with the light's color
    pub fn generate(&self, pixels: u32, sun: &Option<Light>) -> GeneratedPlanet<Kind> {
        let terrain = Terrain::generate(self);
        terrain.generate_planet(pixels, sun)
    }

    /// Convience method to calculate the origin of a planet if it orbited in an exact circle at `distance`
    pub fn set_origin_by_angle(&mut self, angle: Angle<f32>, distance: Length<f32, Kilometers>) {
        self.origin = calculate_origin(angle, distance);
    }
}

pub fn calculate_origin(
    angle: Angle<f32>,
    distance: Length<f32, Kilometers>,
) -> Point2D<f32, Kilometers> {
    let rotation = Rotation2D::new(angle);
    rotation.transform_point(Point2D::from_lengths(distance, Default::default()))
}

/// Structure representing a star projecting light. It is not scientific
pub struct Light {
    /// The color of the light. In most cases, you should use a color close to white.
    pub color: Srgb<f32>,

    /// The intensity of the light. Because the simulation of light isn't scientific,
    /// this is meant to be a multiplicative factor based on the "feel" of how bright
    /// an Earth-like planet appears at Earth-like distances.
    pub sols: f32,
}

impl Default for Light {
    fn default() -> Self {
        Light {
            color: Srgb::new(1., 1., 1.),
            sols: 1.,
        }
    }
}

impl Light {
    pub fn from_u8(red: u8, green: u8, blue: u8, sols: f32) -> Self {
        Self {
            color: Srgb::new(red, green, blue).into_format(),
            sols,
        }
    }
}
