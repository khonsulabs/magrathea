use std::ops::Range;

use euclid::{Angle, Length, Point2D, Rotation2D};
use palette::{Shade, Srgb};
use rand::{rngs::SmallRng, Rng};
use sorted_vec::partial::SortedVec;
use spade::rtree::RTree;

use crate::{
    coloring::ElevationColor,
    planet::Light,
    planet::Planet,
    terrain_point::{TerrainLocation, TerrainPoint},
    types::{Kilometers, LinearRgb},
};

const COLOR_LIGHTEN_DELTA: f32 = 0.1;

pub struct Terrain<Kind> {
    pub surface_chaos: f32,
    pub origin: Point2D<f32, Kilometers>,
    pub radius: Length<f32, Kilometers>,
    pub points: RTree<TerrainPoint>,
    pub elevations: SortedVec<ElevationColor<Kind>>,
}

impl<Kind> Terrain<Kind>
where
    Kind: Clone,
{
    pub fn generate(planet: &Planet<Kind>, rng: &mut SmallRng) -> Self {
        let min_elevation = planet.colors.first().unwrap().elevation.0;
        let max_elevation = planet.colors.last().unwrap().elevation.0;

        // Allow up to a 30% overextension of elevation
        let elevation_variance: f32 = (max_elevation - min_elevation) * rng.gen::<f32>() * 0.3;
        let elevation_range =
            (min_elevation - elevation_variance)..(max_elevation + elevation_variance);

        // How much variation in elevation do we want to allow per kilometer of distance?
        let surface_chaos = rng.gen_range(1.0f32, 2.0);

        let mut terrain = Terrain {
            origin: planet.origin,
            radius: planet.radius,
            points: RTree::new(),
            surface_chaos,
            elevations: Self::generate_elevations(&planet.colors, &elevation_range, rng),
        };

        let terrain_complexity = rng.gen_range(50, 1000);

        for _ in 0..terrain_complexity {
            let x = rng.gen_range(-planet.radius.get(), planet.radius.get());
            let y = rng.gen_range(-planet.radius.get(), planet.radius.get());
            let location = TerrainLocation::new(Point2D::new(x, y));
            let acceptable_range =
                if let Some(neighbor) = terrain.points.nearest_neighbor(&location) {
                    let distance = location.point.distance_to(neighbor.location.point);
                    (neighbor.elevation.0 - distance * surface_chaos)
                        ..(neighbor.elevation.0 + distance * surface_chaos)
                } else {
                    -surface_chaos..surface_chaos
                };
            terrain.points.insert(TerrainPoint {
                location,
                elevation: Kilometers::new(rng.gen_range(
                    acceptable_range.start.max(elevation_range.start),
                    acceptable_range.end.min(elevation_range.end),
                )),
            });
        }

        terrain
    }

    /// Take an ordered list of elevations and their colors, and create a gradient of colors
    /// with ranges of elevations spanning `elevation_range`
    fn generate_elevations(
        colorings: &SortedVec<ElevationColor<Kind>>,
        elevation_range: &Range<f32>,
        rng: &mut SmallRng,
    ) -> SortedVec<ElevationColor<Kind>> {
        let mut elevations = SortedVec::with_capacity(colorings.capacity());
        let mut carryover = 0f32;

        for (index, coloring) in colorings.iter().enumerate() {
            let base = coloring.color.into_linear();
            let elevation = coloring.elevation.0 - carryover;
            if index == 0 {
                // Start
                Self::generate_elevation_range_into(
                    &coloring.kind,
                    elevation_range.start..elevation,
                    base.darken(COLOR_LIGHTEN_DELTA),
                    rng,
                    &mut elevations,
                );
            }

            let lighter = base.lighten(COLOR_LIGHTEN_DELTA);
            if index + 1 == colorings.len() {
                // End
                Self::generate_elevation_range_into(
                    &coloring.kind,
                    elevation..elevation_range.end,
                    lighter,
                    rng,
                    &mut elevations,
                );
            } else {
                let next = colorings[index + 1].elevation.0;
                let end = rng.gen_range(elevation, next);
                Self::generate_elevation_range_into(
                    &coloring.kind,
                    elevation..end,
                    base,
                    rng,
                    &mut elevations,
                );
                carryover = next - end;
            }
        }

        elevations
    }

    /// Generate shades of colors
    fn generate_elevation_range_into(
        kind: &Kind,
        elevation_range: Range<f32>,
        base_color: LinearRgb,
        rng: &mut SmallRng,
        output: &mut SortedVec<ElevationColor<Kind>>,
    ) {
        // Subdivide this range randomly into 3 bands of coloring
        let midpoint = rng.gen_range(elevation_range.start, elevation_range.end);
        let midpoint_color = rng.gen_range(0., COLOR_LIGHTEN_DELTA);

        output.insert(ElevationColor {
            kind: kind.clone(),
            color: Srgb::from_linear(base_color),
            elevation: Kilometers::new(elevation_range.start),
        });
        output.insert(ElevationColor {
            kind: kind.clone(),
            color: Srgb::from_linear(base_color.lighten(midpoint_color)),
            elevation: Kilometers::new(midpoint),
        });
        output.insert(ElevationColor {
            kind: kind.clone(),
            color: Srgb::from_linear(base_color.lighten(COLOR_LIGHTEN_DELTA)),
            elevation: Kilometers::new(elevation_range.end),
        });
    }

    pub fn extrapolate_point(
        &self,
        planet_point: Point2D<f32, Kilometers>,
        sun: &Option<Light>,
    ) -> (Kind, Srgb<u8>) {
        let nearest_points = self
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

        let closest_elevation = match self.elevations.binary_search_by(|probe| {
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
                    let delta_a = self.elevations[index].elevation - extrapolated_elevation;
                    let delta_b = extrapolated_elevation - self.elevations[index - 1].elevation;
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
}
