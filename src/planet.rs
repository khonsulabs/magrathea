use std::{collections::HashMap, hash::Hash};

use crate::{
    coloring::ElevationColor,
    terrain::Terrain,
    types::{Kilometers, Pixels},
};
use euclid::{Angle, Length, Point2D, Rotation2D, Vector2D};
use palette::Srgb;
use sorted_vec::partial::SortedVec;
use uuid::Uuid;

/// A Procedural Planet definition
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
    pub terrain: Terrain<Kind>,
    pub stats: HashMap<Kind, u32>,
}

impl<Kind> Planet<Kind>
where
    Kind: Clone + Hash + Eq,
{
    /// Generates an image of `pixels` wide, and `pixels` tall. If a light is provided
    /// a shadow is simulated, and the colors are mixed with the light's color
    pub fn generate(&self, pixels: u32, sun: &Option<Light>) -> GeneratedPlanet<Kind> {
        let terrain = Terrain::generate(self);

        let mut image = image::RgbaImage::new(pixels, pixels);
        let radius = Length::<f32, Pixels>::new(pixels as f32 / 2.);
        let planet_scale = self.radius / radius;

        let center = Point2D::from_lengths(radius, radius);
        let mut stats = HashMap::new();

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let point = Point2D::new(x as f32, y as f32);
            let distance = point.distance_to(center);

            let planet_point =
                point * planet_scale - Vector2D::from_lengths(self.radius, self.radius);

            let color = if distance < radius.get() {
                let (kind, color) = terrain.extrapolate_point(planet_point, sun);
                // Inside the boundaries of the planet
                let delta = radius.get() - distance;
                let alpha = if delta < 1. {
                    (255. * delta) as u8
                } else {
                    255
                };

                stats
                    .entry(kind)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);

                [color.red as u8, color.green as u8, color.blue as u8, alpha]
            } else {
                Default::default()
            };

            *pixel = image::Rgba(color);
        }

        GeneratedPlanet {
            image,
            terrain,
            stats,
        }
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
