#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Elevation(pub f32);
impl std::ops::Deref for Elevation {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Elevation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl spade::PointN for Elevation {
    type Scalar = f32;

    fn dimensions() -> usize {
        1
    }

    fn from_value(value: Self::Scalar) -> Self {
        Elevation(value)
    }

    fn nth(&self, _index: usize) -> &Self::Scalar {
        &self.0
    }

    fn nth_mut(&mut self, _index: usize) -> &mut Self::Scalar {
        &mut self.0
    }
}
