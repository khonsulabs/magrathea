use crate::{Coloring, Kilometers};
use euclid::{Length, Point2D};
use sorted_vec::partial::SortedVec;
use uuid::Uuid;

pub struct PlanetDefinition {
    pub seed: Uuid,
    pub origin: Point2D<f32, Kilometers>,
    pub radius: Length<f32, Kilometers>,
    pub colors: SortedVec<Coloring>,
}
