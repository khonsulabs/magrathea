use palette::encoding::Linear;
use palette::rgb::Rgb;

pub struct Kilometers;
pub struct Pixels;

pub type LinearRgb = Rgb<Linear<palette::encoding::Srgb>, f32>;
