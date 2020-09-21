use euclid::{Length, Point2D, Rotation2D, Vector2D};
use palette::{encoding::Linear, Shade};
use palette::{rgb::Rgb, Srgb};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sorted_vec::partial::SortedVec;
use uuid::Uuid;

pub struct Kilometers;
pub struct Pixels;

type LinearRgb = Rgb<Linear<palette::encoding::Srgb>, f32>;

mod coloring;
mod elevation;
mod planet;
mod terrain;
mod terrain_point;

use self::{coloring::*, elevation::*, planet::*, terrain::*, terrain_point::*};

fn extrapolate_point(
    planet_point: Point2D<f32, Kilometers>,
    terrain: &Terrain,
    sun: &Light,
    rng: &mut SmallRng,
) -> Srgb<u8> {
    let nearest_point = terrain
        .points
        .nearest_neighbor(&TerrainLocation::new(planet_point))
        .unwrap();

    let (closest_elevation, matched) = match terrain.elevations.binary_search_by(|probe| {
        probe
            .elevation
            .partial_cmp(&nearest_point.elevation)
            .unwrap()
    }) {
        Ok(index) => (index, true),
        Err(index) => (index, false),
    };

    let terrain_color = if matched || closest_elevation == 0 {
        terrain.elevations[closest_elevation].color.into_linear()
    } else if closest_elevation == terrain.elevations.len() {
        terrain.elevations[closest_elevation - 1]
            .color
            .into_linear()
    } else {
        // Randomize the pixel based on a weight of how close it is to each elevation
        let (base_elevation, upper_elevation) = if closest_elevation + 1 == terrain.elevations.len()
        {
            (closest_elevation - 1, closest_elevation)
        } else {
            (closest_elevation, closest_elevation + 1)
        };
        let base_elevation = terrain.elevations[base_elevation];
        let upper_elevation = terrain.elevations[upper_elevation];

        let distance_to_base = (base_elevation.elevation.0 - nearest_point.elevation.0).abs();
        let distance_to_upper = (upper_elevation.elevation.0 - base_elevation.elevation.0).abs();

        if rng.gen::<f32>() * (distance_to_upper + distance_to_base) < distance_to_base {
            let linear = base_elevation.color.into_linear();
            linear.darken(distance_to_base.max(100.) / 1000.)
        } else {
            let linear = upper_elevation.color.into_linear();
            linear.lighten(distance_to_upper.max(100.) / 1000.)
        }
    };

    let space_point = terrain.origin + planet_point.to_vector();
    let angle_to_sun = space_point.to_vector().angle_from_x_axis();
    let distance_to_sun = space_point.distance_to(Default::default());
    let focus_point = Rotation2D::new(angle_to_sun)
        .transform_point(Point2D::from_lengths(terrain.radius, Default::default()));
    let distance_from_focus = planet_point.distance_to(focus_point);

    // Shade based on the lighting
    let distance_dimming = 1.0 - 1. / distance_to_sun;
    let sphere_dimming = distance_from_focus / (terrain.radius.get() * 1.4);
    let sun_base_factor = sun.sols * distance_dimming * sphere_dimming;

    let lit_by_sun = terrain_color
        * sun
            .color
            .into_linear()
            // .darken(1.0 - sun_intensity)
            .darken(sun_base_factor.min(1.0));

    Srgb::from_linear(lit_by_sun).into_format::<u8>()
}

pub struct Light {
    color: Srgb<f32>,
    sols: f32,
}

fn generate_planet(pixels: u32, planet: &PlanetDefinition, sun: &Light) -> image::RgbaImage {
    let mut rng = SmallRng::from_seed(*planet.seed.as_bytes());
    let terrain = Terrain::generate(planet, &mut rng);

    let mut image = image::RgbaImage::new(pixels, pixels);
    let radius = Length::<f32, Pixels>::new(pixels as f32 / 2.);
    let planet_scale = planet.radius / radius;

    let center = Point2D::from_lengths(radius, radius);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let point = Point2D::new(x as f32, y as f32);
        let distance = point.distance_to(center);

        let planet_point =
            point * planet_scale - Vector2D::from_lengths(planet.radius, planet.radius);

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

fn generate() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let earth_distance = Length::<f32, Kilometers>::new(150_200_000.);
    let earth_radius = Length::new(6_371.);

    let mut rng = rand::thread_rng();
    let radius_factor = rng.gen_range(0.2f32, 2.);

    let radius = earth_radius * radius_factor;
    let rotation = Rotation2D::radians(rng.gen_range(0., std::f32::consts::PI * 2.));
    let origin =
        rotation.transform_point(Point2D::from_lengths(earth_distance, Default::default()));

    let image = generate_planet(
        (65. * radius_factor) as u32,
        &PlanetDefinition {
            seed: Uuid::new_v4(),
            origin,
            radius,
            colors: SortedVec::from_unsorted(vec![
                Coloring {
                    color: Srgb::new(0., 0., 1.),
                    elevation: Elevation(0.),
                },
                Coloring {
                    color: Srgb::new(0., 1., 0.),
                    elevation: Elevation(500.),
                },
                Coloring {
                    color: Srgb::new(1., 1., 0.),
                    elevation: Elevation(900.),
                },
            ]),
        },
        &Light {
            color: Srgb::new(1., 1., 1.),
            sols: 1.,
        },
    );
    let end = std::time::Instant::now();
    println!("Elapsed: {:?}", end.checked_duration_since(start));
    image.save("output.png")?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    loop {
        generate()?;

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
