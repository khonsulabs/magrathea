use crate::{
    coloring::Coloring,
    planet::{Light, Planet},
    types::Kilometers,
};
use std::path::PathBuf;
use structopt::StructOpt;
use uuid::Uuid;

use euclid::{Angle, Length};
use palette::Srgb;

#[derive(Debug, StructOpt)]
#[structopt(name = "magrathea", about = "A pixel-art planet generator")]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, StructOpt, PartialEq, Clone)]
pub enum PlanetCommand {
    New(NewPlanetOptions),
}

impl Default for PlanetCommand {
    fn default() -> Self {
        PlanetCommand::New(NewPlanetOptions::default())
    }
}

#[derive(Debug, StructOpt, PartialEq)]
pub enum Command {
    #[cfg(feature = "editor")]
    Edit(Edit),
    Generate(Generate),
}

#[cfg(feature = "editor")]
impl Default for Command {
    fn default() -> Self {
        Command::Edit(Edit::default())
    }
}

#[cfg(not(feature = "editor"))]
impl Default for Command {
    fn default() -> Self {
        Command::Generate(Generate::default())
    }
}

#[derive(Debug, StructOpt, PartialEq, Default)]
pub struct Edit {
    /// Render resolution, in pixels
    #[structopt(short = "p", long)]
    pub resolution: Option<u32>,

    #[structopt(subcommand)]
    pub command: Option<PlanetCommand>,

    /// Simulate sun lighting, using the hexadecimal color.
    pub sun_color: Option<String>,

    /// If simulating the sun, how intense should the light be?
    pub sols: Option<f32>,
}

#[derive(Debug, Default, StructOpt, PartialEq)]
pub struct Generate {
    #[structopt(short, long)]
    pub output: Option<PathBuf>,

    /// Render resolution, in pixels
    #[structopt(short = "p", long)]
    pub resolution: Option<u32>,

    /// Repeat duration, in seconds
    #[structopt(short, long)]
    pub repeat: Option<f32>,

    #[structopt(subcommand)]
    pub command: Option<PlanetCommand>,

    /// Simulate sun lighting, using the hexadecimal color.
    pub sun_color: Option<String>,

    /// If simulating the sun, how intense should the light be?
    pub sols: Option<f32>,
}

pub trait Lightable {
    fn sun_color_hex(&self) -> &'_ Option<String>;
    fn sols(&self) -> &'_ Option<f32>;

    fn sun_color(&self) -> Option<Srgb<u8>> {
        self.sun_color_hex().as_ref().map(|hex_color| {
            let bytes = hex::decode(hex_color)
                .expect("Only 6-character hexadecimal codes are allowed, e.g., FF1234");
            assert!(
                bytes.len() == 3,
                "Only 6-character hexadecimal codes are allowed, e.g., FF1234"
            );

            Srgb::new(bytes[0], bytes[1], bytes[2])
        })
    }

    fn light(&self) -> Option<Light> {
        self.sun_color().map(|color| Light {
            color: color.into_format(),
            sols: self.sols().unwrap_or(1.),
        })
    }
}

impl Lightable for Edit {
    fn sun_color_hex(&self) -> &'_ Option<String> {
        &self.sun_color
    }

    fn sols(&self) -> &'_ Option<f32> {
        &self.sols
    }
}

impl Lightable for Generate {
    fn sun_color_hex(&self) -> &'_ Option<String> {
        &self.sun_color
    }

    fn sols(&self) -> &'_ Option<f32> {
        &self.sols
    }
}

#[derive(Debug, StructOpt, PartialEq, Default, Clone)]
pub struct NewPlanetOptions {
    /// Planet distance from sun, in kilometers
    #[structopt(short, long)]
    distance: Option<f32>,

    /// Planet rotation around the sun, in radians
    #[structopt(short, long)]
    angle: Option<f32>,

    /// Planet's radius, in kilometers
    #[structopt(short, long)]
    radius: Option<f32>,
}

impl Into<Planet> for NewPlanetOptions {
    fn into(self) -> Planet {
        let distance = Length::<f32, Kilometers>::new(self.distance.unwrap_or(150_200_000.));
        let radius = Length::new(self.radius.unwrap_or(6_371.));
        let origin =
            Planet::calculate_origin(Angle::radians(self.angle.unwrap_or(-2.35619)), distance);
        Planet {
            seed: Uuid::new_v4(),
            origin,
            radius,
            colors: Coloring::earthlike(),
        }
    }
}
