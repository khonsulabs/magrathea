# Magrathea

[![crate version](https://img.shields.io/crates/v/magrathea.svg)](https://crates.io/crates/magrathea)

![Example Output from 9/21/2020](./Example.png)

> And thus were created the conditions for a staggering new form of specialist industry: custom-made luxury planet building. The home of this industry was the planet Magrathea, where hyperspatial engineers sucked matter through white holes in space to form it into dream planets—gold planets, platinum planets, soft rubber planets with lots of earthquakes—all lovingly made to meet the exacting standards that the Galaxy’s richest men naturally came to expect. _[From the Hitchhiker's Guide to the Galaxy](https://hitchhikers.fandom.com/wiki/Magrathea)_

Magrathea is a procedural planet generator, focused on generating pixel-art style planets. It is written in [Rust](https://rust-lang.org) and can be used either as a standalone executable or as a crate.

## Using from the command line

This will print the help output for the command:

`cargo run --features cli -h`

If you wish to also see commands relating to launching the GUI editor, pass the `editor` feature instead of `cli`:

`cargo run --features editor -h`

### Examples

#### Generate a random 128x128 pixel into `./planet.png`

`cargo run --features cli generate -o ./planet.png`

#### Run the editor with a new planet

`cargo run --features editor edit`

## Using as a crate

Add magrathea to your Cargo.toml:

`magrathea = "0.0.1"`

Create a 128x128 rendering of a random Planet:

```rust
let planet = magrathea::Planet {
    seed: Uuid::new_v4(),
    origin: Point2D::new(x_km, y_km),
    radius: Length::new(radius_km),
    colors: Coloring::earthlike(),
};
let image = planet.generate(128, &Light::defaulFt())
```

## Future Development

This is being developed for use in a game that is TBA, by [Khonsu Labs](https://khonsulabs.com/). Stability for this crate means being able to input the same values into the generation functions and receive the same outputs. As such, any minor version upgrade (e.g., 0.1 to 0.2) will be done whenever any changes break existing seed compatibility. Until v0.1, however, **no stability is guaranteed between updates**.

## License

Magrathea is licensed under the [MIT License](./LICENSE.txt).
