use crate::{
    coloring::ElevationColor,
    planet::{GeneratedPlanet, Light, Planet},
    types::{Kilometers, Pixels},
};
use euclid::{Angle, Length, Point2D, Rotation2D, Vector2D};
use noise::{NoiseFn, OpenSimplex, Seedable};
use palette::{Shade, Srgb};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sorted_vec::partial::SortedVec;
use std::{collections::HashMap, hash::Hash};

/// A randomly generated elevation map
pub struct Terrain<Kind> {
    /// Per kilometer of distance between another point, how much can the surface change?
    pub surface_chaos: f32,

    /// The origin of the planet
    pub origin: Point2D<f32, Kilometers>,

    /// The radius of the planet
    pub radius: Length<f32, Kilometers>,

    /// A 2d spatial tree of points
    pub noise: OpenSimplex,

    /// A sorted collection of ElevationColors
    pub elevations: SortedVec<ElevationColor<Kind>>,
}

impl<Kind> Terrain<Kind>
where
    Kind: Clone + Hash + Eq,
{
    /// Randomly generate a new terrain for the Planet provided
    pub fn generate(planet: &Planet<Kind>) -> Self {
        let mut rng = SmallRng::seed_from_u64(planet.seed);

        // How much variation in elevation do we want to allow per kilometer of distance?
        let surface_chaos = rng.gen_range(1.0f32..planet.max_chaos.max(1.));
        let terrain_seed = rng.gen();

        Terrain {
            origin: planet.origin,
            radius: planet.radius,
            noise: OpenSimplex::new().set_seed(terrain_seed),
            surface_chaos,
            elevations: planet.colors.clone(),
        }
    }

    /// For a given point on the surface, return what kind and what color the point is
    pub fn extrapolate_point(
        &self,
        planet_point: Point2D<f32, Kilometers>,
        sun: &Option<Light>,
    ) -> (Kind, Srgb<u8>) {
        let normalized_point = planet_point.to_vector() / self.radius.get() * self.surface_chaos;
        let noise = self.noise.get(normalized_point.to_f64().to_array()) as f32;
        // Convert the -1.0..1.0 range of the noise to 0.0..1.0
        let noise = (noise + 1.0) / 2.0;
        let elevation_range =
            self.elevations.first().unwrap().elevation..self.elevations.last().unwrap().elevation;
        let elevation =
            elevation_range.start + (elevation_range.end - elevation_range.start) * noise;

        let closest_elevation = match self
            .elevations
            .binary_search_by(|probe| probe.elevation.partial_cmp(&elevation).unwrap())
        {
            Ok(index) => index,
            Err(index) => {
                // We didn't match, generate a random variation between these two elevations with probabilty from how close of a match it is
                if index == 0 {
                    index
                } else {
                    let delta_a = self.elevations[index].elevation - elevation;
                    let delta_b = elevation - self.elevations[index - 1].elevation;
                    if delta_a > delta_b {
                        index
                    } else {
                        index - 1
                    }
                }
            }
        };

        let terrain_kind = self.elevations[closest_elevation].kind.clone();
        let terrain_color = self.elevations[closest_elevation].color.into_linear();

        let space_point = self.origin + planet_point.to_vector();
        let angle_to_sun =
            Angle::radians(space_point.y.atan2(space_point.x)) + Angle::degrees(180.);
        let distance_to_sun = space_point.distance_to(Default::default());
        let focus_point = Rotation2D::new(angle_to_sun)
            .transform_point(Point2D::from_lengths(self.radius, Default::default()));
        let distance_from_focus = planet_point.distance_to(focus_point);

        // Shade based on the lighting
        let color = match sun {
            Some(sun) => {
                let distance_dimming = 1.0 - 1. / distance_to_sun;
                let sphere_dimming = distance_from_focus / (self.radius.get() * 1.4);
                let sun_base_factor = sun.sols * distance_dimming * sphere_dimming;

                terrain_color
                    * sun
                        .color
                        .into_linear()
                        // .darken(1.0 - sun_intensity)
                        .darken(sun_base_factor.min(1.0))
            }
            None => terrain_color,
        };

        let color = Srgb::from_linear(color);
        (
            terrain_kind,
            Srgb::new(
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
            ),
        )
    }

    /// Generates an image of `pixels` wide, and `pixels` tall. If a light is provided
    /// a shadow is simulated, and the colors are mixed with the light's color
    pub fn generate_planet(self, pixels: u32, sun: &Option<Light>) -> GeneratedPlanet<Kind> {
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
                let (kind, color) = self.extrapolate_point(planet_point, sun);
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

        GeneratedPlanet { image, stats }
    }
}
