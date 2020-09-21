use std::ops::Range;

use euclid::{Length, Point2D};
use palette::{Shade, Srgb};
use rand::{rngs::SmallRng, Rng};
use sorted_vec::partial::SortedVec;
use spade::rtree::RTree;

use crate::{
    coloring::Coloring,
    elevation::Elevation,
    planet::PlanetDefinition,
    terrain_point::{TerrainLocation, TerrainPoint},
    Kilometers, LinearRgb,
};

const COLOR_LIGHTEN_DELTA: f32 = 0.1;

pub struct Terrain {
    pub origin: Point2D<f32, Kilometers>,
    pub radius: Length<f32, Kilometers>,
    pub points: RTree<TerrainPoint>,
    pub elevations: SortedVec<Coloring>,
}

impl Terrain {
    pub fn generate(planet: &PlanetDefinition, rng: &mut SmallRng) -> Self {
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
            elevations: Self::generate_elevations(&planet.colors, &elevation_range, rng),
        };

        let terrain_complexity = rng.gen_range(10, 1000);

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
                elevation: Elevation(rng.gen_range(
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
        colorings: &SortedVec<Coloring>,
        elevation_range: &Range<f32>,
        rng: &mut SmallRng,
    ) -> SortedVec<Coloring> {
        let mut elevations = SortedVec::with_capacity(colorings.capacity());
        let mut carryover = 0f32;

        for (index, coloring) in colorings.iter().enumerate() {
            let base = coloring.color.into_linear();
            let elevation = coloring.elevation.0 - carryover;
            if index == 0 {
                // Start
                Self::generate_elevation_range_into(
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
                    elevation..elevation_range.end,
                    lighter,
                    rng,
                    &mut elevations,
                );
            } else {
                let next = colorings[index + 1].elevation.0;
                let end = rng.gen_range(elevation, next);
                Self::generate_elevation_range_into(elevation..end, base, rng, &mut elevations);
                carryover = next - end;
            }
        }

        elevations
    }

    /// Generate shades of colors
    fn generate_elevation_range_into(
        elevation_range: Range<f32>,
        base_color: LinearRgb,
        rng: &mut SmallRng,
        output: &mut SortedVec<Coloring>,
    ) {
        // Subdivide this range randomly into 3 bands of coloring
        let midpoint = rng.gen_range(elevation_range.start, elevation_range.end);
        let midpoint_color = rng.gen_range(0., COLOR_LIGHTEN_DELTA);

        output.insert(Coloring {
            color: Srgb::from_linear(base_color),
            elevation: Elevation(elevation_range.start),
        });
        output.insert(Coloring {
            color: Srgb::from_linear(base_color.lighten(midpoint_color)),
            elevation: Elevation(midpoint),
        });
        output.insert(Coloring {
            color: Srgb::from_linear(base_color.lighten(COLOR_LIGHTEN_DELTA)),
            elevation: Elevation(elevation_range.end),
        });
    }
}
