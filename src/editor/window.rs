use euclid::{Point2D, Rotation2D};
use kludgine::prelude::*;
use palette::Srgb;
use uuid::Uuid;

use crate::{coloring::Coloring, planet::Light, planet::PlanetDefinition, Kilometers};

pub struct EditorWindow {
    planet: PlanetDefinition,
    resolution: u32,
    image: Entity<Image>,
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "magrathea", about = "A pixel-art planet generator")]
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

    /// Render resolution, in pixels
    #[structopt(short = "p", long)]
    resolution: Option<u32>,
}

impl EditorWindow {
    pub fn new(options: NewPlanetOptions) -> Self {
        let distance = Length::<f32, Kilometers>::new(options.distance.unwrap_or(150_200_000.));
        let radius = Length::new(options.radius.unwrap_or(6_371.));
        //-2.356
        let origin = Self::calculate_origin(Angle::radians(options.angle.unwrap_or(0.)), distance);

        let resolution = options.resolution.unwrap_or(128);

        Self {
            planet: PlanetDefinition {
                seed: Uuid::new_v4(),
                origin,
                radius,
                colors: Coloring::earthlike(),
            },
            resolution,
            image: Default::default(),
        }
    }

    async fn generate_image(&self) -> Sprite {
        let image = self.planet.generate(
            self.resolution,
            &Light {
                color: Srgb::new(1., 1., 1.),
                sols: 1.,
            },
        );

        Sprite::single_frame(Texture::new(image::DynamicImage::ImageRgba8(image))).await
    }

    fn calculate_origin(
        angle: Angle,
        distance: Length<f32, Kilometers>,
    ) -> Point2D<f32, Kilometers> {
        let rotation = Rotation2D::new(angle);
        rotation.transform_point(Point2D::from_lengths(distance, Default::default()))
    }

    async fn regenerate_image(&self) {
        let _ = self
            .image
            .send(ImageCommand::SetSprite(self.generate_image().await))
            .await;
    }
}

impl Window for EditorWindow {}

impl WindowCreator for EditorWindow {
    fn window_title() -> String {
        "Magrathea".to_owned()
    }
}

#[derive(Clone, Debug)]
pub enum EditorMessage {
    NewSeed,
}

#[async_trait]
impl InteractiveComponent for EditorWindow {
    type Message = EditorMessage;
    type Command = ();
    type Event = ();

    async fn receive_message(
        &mut self,
        context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            EditorMessage::NewSeed => {
                self.planet.seed = Uuid::new_v4();
                self.regenerate_image().await;
                context.set_needs_redraw().await;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Component for EditorWindow {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let planet = self.generate_image().await;
        self.image = self
            .new_entity(
                context,
                Image::new(planet)
                    .options(ImageOptions::default().scaling(ImageScaling::AspectFit)),
            )
            .bounds(AbsoluteBounds {
                left: Dimension::from_f32(64.),
                top: Dimension::from_f32(64.),
                right: Dimension::from_f32(64.),
                bottom: Dimension::from_f32(64.),
                ..Default::default()
            })
            .interactive(false)
            .insert()
            .await?;

        self.new_entity(context, Button::new("Randomize Seed"))
            .bounds(AbsoluteBounds {
                right: Dimension::from_f32(10.),
                bottom: Dimension::from_f32(10.),
                ..Default::default()
            })
            .callback(|_| EditorMessage::NewSeed)
            .insert()
            .await?;
        Ok(())
    }

    async fn mouse_down(
        &mut self,
        context: &mut Context,
        window_position: Point<f32, Scaled>,
        _button: MouseButton,
    ) -> KludgineResult<EventStatus> {
        let radius = self.planet.origin.distance_to(Point::default());
        let bounds = context.last_layout().await.inner_bounds();
        let click_relative_to_center = window_position - bounds.center().to_vector();
        let angle = Angle::radians(click_relative_to_center.y.atan2(click_relative_to_center.x));
        self.planet.origin = Self::calculate_origin(angle, Length::new(radius));

        self.regenerate_image().await;

        context.set_needs_redraw().await;

        Ok(EventStatus::Processed)
    }

    async fn update(&mut self, _context: &mut SceneContext) -> KludgineResult<()> {
        Ok(())
    }
}
