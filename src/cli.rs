use crate::{coloring::Earthlike, planet::Planet};
use std::path::PathBuf;
use structopt::StructOpt;

pub(crate) mod args;

use args::{Args, Command, Generate, Lightable, PlanetCommand};

pub fn run() -> anyhow::Result<()> {
    let args = Args::from_args();
    match args.command.unwrap_or_default() {
        #[cfg(feature = "editor")]
        Command::Edit(edit) => crate::editor::run(edit),
        Command::Generate(command) => generate(command),
    }
}

fn generate(options: Generate) -> anyhow::Result<()> {
    loop {
        let planet: Planet<Earthlike> = match options.command.clone().unwrap_or_default() {
            PlanetCommand::New(planet_options) => planet_options.into(),
        };

        let generated = planet.generate(options.resolution.unwrap_or(128), &options.light());

        generated.image.save(
            options
                .output
                .clone()
                .unwrap_or_else(|| PathBuf::from("output.png")),
        )?;

        println!("Generated image stats: {:#?}", generated.stats);

        if let Some(seconds) = options.repeat {
            std::thread::sleep(std::time::Duration::from_secs_f32(seconds));
        } else {
            break;
        }
    }

    Ok(())
}
