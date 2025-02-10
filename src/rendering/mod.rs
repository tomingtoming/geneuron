use ggez::{Context, GameResult};
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, Text};
use crate::world::World;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,  // Make zoom field public
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 1.0,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_size = (width, height);
    }

    pub fn render(&self, ctx: &mut Context, world: &World) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        
        // Apply zoom based on window size
        canvas.set_screen_coordinates(graphics::Rect::new(
            0.0, 
            0.0, 
            self.window_size.0 / self.zoom, 
            self.window_size.1 / self.zoom,
        ));

        // Draw food sources
        for food in &world.food_manager.foods {
            let food_circle = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food.position.x, food.position.y],
                food.size,
                0.1,
                food.color,
            )?;
            canvas.draw(&food_circle, DrawParam::default());
        }

        // Draw creatures
        for creature in &world.creatures {
            // Creature body
            let body = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [creature.physics.position.x, creature.physics.position.y],
                10.0,
                0.1,
                creature.color,
            )?;
            canvas.draw(&body, DrawParam::default());

            // Direction indicator with mode color
            let direction_line = Mesh::new_line(
                ctx,
                &[
                    [creature.physics.position.x, creature.physics.position.y],
                    [
                        creature.physics.position.x + 20.0 * creature.physics.rotation.cos(),
                        creature.physics.position.y + 20.0 * creature.physics.rotation.sin()
                    ],
                ],
                2.0,
                creature.mode_color,
            )?;
            canvas.draw(&direction_line, DrawParam::default());
        }

        // Display simulation information
        let info_text = Text::new(format!(
            "Generation: {}\nCreatures: {}\nElapsed Time: {:.1}s\nFPS: {:.1}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            ctx.time.fps(),
        ));
        canvas.draw(
            &info_text,
            DrawParam::default()
                .color(Color::WHITE)
                .dest([10.0, 10.0]),
        );

        canvas.finish(ctx)?;
        Ok(())
    }
}