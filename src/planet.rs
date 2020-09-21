use crate::{
    coloring::ElevationColor,
    terrain::Terrain,
    terrain_point::TerrainLocation,
    types::{Kilometers, Pixels},
};
use euclid::{Angle, Length, Point2D, Rotation2D, Vector2D};
use palette::Shade;
use palette::Srgb;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sorted_vec::partial::SortedVec;
use uuid::Uuid;

/// A Procedural Planet definition
pub struct Planet {
    /// The unique value that is used to seed the random number generator
    pub seed: Uuid,

    /// The origin of the planet relative to the star it orbits
    pub origin: Point2D<f32, Kilometers>,

    /// The radius of the planet
    pub radius: Length<f32, Kilometers>,

    /// The ElevationColors used to generate the terrain
    pub colors: SortedVec<ElevationColor>,
}

fn extrapolate_point(
    planet_point: Point2D<f32, Kilometers>,
    terrain: &Terrain,
    sun: &Option<Light>,
    rng: &mut SmallRng,
) -> Srgb<u8> {
    let nearest_points = terrain
        .points
        .nearest_n_neighbors(&TerrainLocation::new(planet_point), 3);
    assert!(nearest_points.len() == 3);

    let distances = nearest_points
        .iter()
        .map(|p| p.location.point.distance_to(planet_point))
        .collect::<Vec<_>>();
    let total_distance = distances.iter().sum::<f32>();

    let extrapolated_elevation = Kilometers::new(
        nearest_points
            .iter()
            .enumerate()
            .map(|(index, point)| distances[index] / total_distance * point.elevation.get())
            .sum(),
    );

    let closest_elevation = match terrain.elevations.binary_search_by(|probe| {
        probe
            .elevation
            .partial_cmp(&extrapolated_elevation)
            .unwrap()
    }) {
        Ok(index) => index,
        Err(index) => {
            // We didn't match, generate a random variation between these two elevations with probabilty from how close of a match it is
            if index == 0 {
                index
            } else {
                let delta_a = terrain.elevations[index].elevation - extrapolated_elevation;
                let delta_b = extrapolated_elevation - terrain.elevations[index - 1].elevation;
                if rng.gen_bool((delta_a / (delta_a + delta_b)).get() as f64) {
                    index
                } else {
                    index - 1
                }
            }
        }
    };

    let terrain_color = terrain.elevations[closest_elevation].color.into_linear();

    let space_point = terrain.origin + planet_point.to_vector();
    let angle_to_sun = Angle::radians(space_point.y.atan2(space_point.x)) + Angle::degrees(180.);
    let distance_to_sun = space_point.distance_to(Default::default());
    let focus_point = Rotation2D::new(angle_to_sun)
        .transform_point(Point2D::from_lengths(terrain.radius, Default::default()));
    let distance_from_focus = planet_point.distance_to(focus_point);

    // Shade based on the lighting
    let color = match sun {
        Some(sun) => {
            let distance_dimming = 1.0 - 1. / distance_to_sun;
            let sphere_dimming = distance_from_focus / (terrain.radius.get() * 1.4);
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
    Srgb::new(
        (color.red * 255.0) as u8,
        (color.green * 255.0) as u8,
        (color.blue * 255.0) as u8,
    )
}

impl Planet {
    /// Generates an image of `pixels` wide, and `pixels` tall. If a light is provided
    /// a shadow is simulated, and the colors are mixed with the light's color
    pub fn generate(&self, pixels: u32, sun: &Option<Light>) -> image::RgbaImage {
        let mut rng = SmallRng::from_seed(*self.seed.as_bytes());
        let terrain = Terrain::generate(self, &mut rng);

        let mut image = image::RgbaImage::new(pixels, pixels);
        let radius = Length::<f32, Pixels>::new(pixels as f32 / 2.);
        let planet_scale = self.radius / radius;

        let center = Point2D::from_lengths(radius, radius);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let point = Point2D::new(x as f32, y as f32);
            let distance = point.distance_to(center);

            let planet_point =
                point * planet_scale - Vector2D::from_lengths(self.radius, self.radius);

            let color = if distance < radius.get() {
                let color = extrapolate_point(planet_point, &terrain, sun, &mut rng);
                // Inside the boundaries of the planet
                let delta = radius.get() - distance;
                let alpha = if delta < 1. {
                    (255. * delta) as u8
                } else {
                    255
                };

                [color.red as u8, color.green as u8, color.blue as u8, alpha]
            } else {
                Default::default()
            };

            *pixel = image::Rgba(color);
        }

        image
    }

    /// Convience method to calculate the origin of a planet if it orbited in an exact circle at `distance`
    pub fn calculate_origin(
        angle: Angle<f32>,
        distance: Length<f32, Kilometers>,
    ) -> Point2D<f32, Kilometers> {
        let rotation = Rotation2D::new(angle);
        rotation.transform_point(Point2D::from_lengths(distance, Default::default()))
    }
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
