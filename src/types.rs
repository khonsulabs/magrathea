use euclid::Length;

/// Unit type for use in euclid geometric types to represent kilometers
#[derive(Clone, Copy, Default)]
pub struct Kilometers;

impl Kilometers {
    pub fn new(km: f32) -> Length<f32, Kilometers> {
        Length::new(km)
    }
}
/// Unit type for use in euclid geometric types to represent pixel measurements
pub(crate) struct Pixels;
