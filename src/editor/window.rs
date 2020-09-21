use kludgine::prelude::*;
use uuid::Uuid;
use std::time::Duration;

use crate::{
    cli::args::{Lightable, Edit, PlanetCommand},
    planet::{Light, Planet}, 
};

pub struct EditorWindow {
    planet: Planet,
    resolution: u32,
    image: Entity<Image>,
    seed_label: Entity<Label>,
    regenerate_throttle: RequiresInitialization<Timeout<Self>>,
    light: Option<Light>,
}

impl EditorWindow {
    pub(crate) fn new(options: Edit) -> Self {
        let light = options.light();
        let planet = match options.command.unwrap_or_default() {
            PlanetCommand::New(options) => options.into(),
        };

        let resolution = options.resolution.unwrap_or(128);
        Self {
            planet,
            resolution,
            image: Default::default(),
            seed_label: Default::default(),
            light,
            regenerate_throttle: Default::default()
        }
    }

    async fn generate_image(&self) -> Sprite {
        let image = self.planet.generate(
            self.resolution,
            &self.light,
        );

        Sprite::single_frame(Texture::new(image::DynamicImage::ImageRgba8(image))).await
    }

    async fn set_origin(&mut self, context: &mut Context, window_position: Point<f32, Scaled>) {
        let radius = self.planet.origin.distance_to(Point::default());
        let bounds = context.last_layout().await.inner_bounds();
        let click_relative_to_center = window_position - bounds.center().to_vector();
        let angle = Angle::radians(click_relative_to_center.y.atan2(click_relative_to_center.x));
        self.planet.origin = Planet::calculate_origin(angle, Length::new(radius));

        self.regenerate_image().await;

        context.set_needs_redraw().await;
    }

    async fn regenerate_image(&self) {
        self.regenerate_throttle.send(EditorCommand::RegenerateImage).await;
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
#[derive(Clone, Debug)]
pub enum EditorCommand {
    RegenerateImage,
}

#[async_trait]
impl InteractiveComponent for EditorWindow {
    type Message = EditorMessage;
    type Command = EditorCommand;
    type Event = ();

    async fn receive_message(
        &mut self,
        context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            EditorMessage::NewSeed => {
                self.planet.seed = Uuid::new_v4();
                let _ = self
                    .seed_label
                    .send(LabelCommand::SetValue(self.planet.seed.to_string()))
                    .await;
                self.regenerate_image().await;
                context.set_needs_redraw().await;
            }
        }

        Ok(())
    }

    async fn receive_input(&mut self, _context: &mut Context, command: Self::Command) -> KludgineResult<()>
    {
        match command {
            EditorCommand::RegenerateImage => {
                let _ = self
                .image
                .send(ImageCommand::SetSprite(self.generate_image().await))
                .await;
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

        self.seed_label = self
            .new_entity(context, Label::new(self.planet.seed.to_string()))
            .bounds(AbsoluteBounds {
                bottom: Dimension::from_f32(10.),
                ..Default::default()
            })
            .style(Style {
                color: Some(Color::WHITE),
                ..Default::default()
            })
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

        self.regenerate_throttle.initialize_with(Timeout::new(Duration::from_millis(50), context.entity()));
        

        Ok(())
    }

    async fn mouse_down(
        &mut self,
        context: &mut Context,
        window_position: Point<f32, Scaled>,
        _button: MouseButton,
    ) -> KludgineResult<EventStatus> {
        self.set_origin(context, window_position).await;

        Ok(EventStatus::Processed)
    }

    async fn mouse_drag(
        &mut self,
        context: &mut Context,
        window_position: Option<Point<f32, Scaled>>,
        _button: MouseButton,
    ) -> KludgineResult<()> {
        if let Some(window_position) = window_position {
            self.set_origin(context, window_position).await;
        }

        Ok(())
    }

    async fn update(&mut self, _context: &mut SceneContext) -> KludgineResult<()> {
        Ok(())
    }
}
