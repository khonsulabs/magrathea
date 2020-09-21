mod coloring;
#[cfg(feature = "editor")]
mod editor;
mod elevation;
mod planet;
mod terrain;
mod terrain_point;
mod types;

use std::path::PathBuf;

use coloring::Coloring;
use euclid::{Angle, Length};
use palette::Srgb;
use planet::{Light, Planet};
#[cfg(feature = "cli")]
use structopt::StructOpt;
pub use types::*;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "magrathea", about = "A pixel-art planet generator")]
struct Args {
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, StructOpt, PartialEq, Clone)]
enum PlanetCommand {
    New(NewPlanetOptions),
}

impl Default for PlanetCommand {
    fn default() -> Self {
        PlanetCommand::New(NewPlanetOptions::default())
    }
}

#[derive(Debug, StructOpt, PartialEq)]
enum Command {
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
struct Edit {
    /// Render resolution, in pixels
    #[structopt(short = "p", long)]
    resolution: Option<u32>,

    #[structopt(subcommand)]
    command: Option<PlanetCommand>,

    /// Simulate sun lighting, using the hexadecimal color.
    sun_color: Option<String>,

    /// If simulating the sun, how intense should the light be?
    sols: Option<f32>,
}

#[derive(Debug, Default, StructOpt, PartialEq)]
struct Generate {
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// Render resolution, in pixels
    #[structopt(short = "p", long)]
    resolution: Option<u32>,

    /// Repeat duration, in seconds
    #[structopt(short, long)]
    repeat: Option<f32>,

    #[structopt(subcommand)]
    command: Option<PlanetCommand>,

    /// Simulate sun lighting, using the hexadecimal color.
    sun_color: Option<String>,

    /// If simulating the sun, how intense should the light be?
    sols: Option<f32>,
}

trait Lightable {
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
            colors: Coloring::sunlike(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    match args.command.unwrap_or_default() {
        #[cfg(feature = "editor")]
        Command::Edit(edit) => editor::run(edit),
        Command::Generate(command) => generate(command),
    }
}

fn generate(options: Generate) -> anyhow::Result<()> {
    loop {
        let planet: Planet = match options.command.clone().unwrap_or_default() {
            PlanetCommand::New(planet_options) => planet_options.into(),
        };

        let image = planet.generate(options.resolution.unwrap_or(128), &options.light());

        image.save(
            options
                .output
                .clone()
                .unwrap_or_else(|| PathBuf::from("output.png")),
        )?;

        if let Some(seconds) = options.repeat {
            std::thread::sleep(std::time::Duration::from_secs_f32(seconds));
        } else {
            break;
        }
    }

    Ok(())
}
