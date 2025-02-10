use crate::world::World;
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh, PxScale, Text};
use ggez::{Context, GameResult};
use nalgebra::Point2;

pub struct Renderer {
    window_size: (f32, f32),
    pub zoom: f32,                    // Make zoom field public
    selected_creature: Option<usize>, // Add selected creature index
    pub camera_offset: Point2<f32>,   // カメラの位置をパブリックに
    following_selected: bool,         // 選択中の生物を追従するかどうか
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        Renderer {
            window_size: (width, height),
            zoom: 0.5, // デフォルトのズームを1.0から0.5に変更（より広い視野）
            selected_creature: None,
            camera_offset: Point2::new(0.0, 0.0),
            following_selected: false,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        // より広い範囲でズーム可能に
        self.zoom = zoom.clamp(0.2, 2.0); // max zoom を5.0から2.0に変更
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        // 古いビューポート範囲を保存
        let old_view_width = self.window_size.0 / self.zoom;
        let old_view_height = self.window_size.1 / self.zoom;
        let old_center_x = self.camera_offset.x + old_view_width / 2.0;
        let old_center_y = self.camera_offset.y + old_view_height / 2.0;

        // ウィンドウサイズを更新
        self.window_size = (width, height);

        // 新しいビューポートの中心を古いビューポートの中心に合わせる
        let new_view_width = width / self.zoom;
        let new_view_height = height / self.zoom;
        self.camera_offset.x = old_center_x - new_view_width / 2.0;
        self.camera_offset.y = old_center_y - new_view_height / 2.0;
    }

    pub fn select_creature(&mut self, index: Option<usize>) {
        self.selected_creature = index;
        if index.is_none() {
            self.following_selected = false;
        }
    }

    pub fn toggle_follow(&mut self) {
        if self.selected_creature.is_some() {
            self.following_selected = !self.following_selected;
        }
    }

    fn update_camera(&mut self, world: &World) {
        if self.following_selected {
            if let Some(selected_idx) = self.selected_creature {
                if let Some(creature) = world.creatures.get(selected_idx) {
                    let view_width = self.window_size.0 / self.zoom;
                    let view_height = self.window_size.1 / self.zoom;

                    // 生物を中心にしたいビューポートの位置を計算
                    let target_x = creature.physics.position.x - view_width / 2.0;
                    let target_y = creature.physics.position.y - view_height / 2.0;

                    // カメラを必要最小限だけ移動させる
                    let dx = if target_x < self.camera_offset.x {
                        let diff = self.camera_offset.x - target_x;
                        if diff > world.world_bounds.0 / 2.0 {
                            world.world_bounds.0 - diff
                        } else {
                            -diff
                        }
                    } else if target_x > self.camera_offset.x {
                        let diff = target_x - self.camera_offset.x;
                        if diff > world.world_bounds.0 / 2.0 {
                            -(world.world_bounds.0 - diff)
                        } else {
                            diff
                        }
                    } else {
                        0.0
                    };

                    let dy = if target_y < self.camera_offset.y {
                        let diff = self.camera_offset.y - target_y;
                        if diff > world.world_bounds.1 / 2.0 {
                            world.world_bounds.1 - diff
                        } else {
                            -diff
                        }
                    } else if target_y > self.camera_offset.y {
                        let diff = target_y - self.camera_offset.y;
                        if diff > world.world_bounds.1 / 2.0 {
                            -(world.world_bounds.1 - diff)
                        } else {
                            diff
                        }
                    } else {
                        0.0
                    };

                    // カメラ位置を更新
                    self.camera_offset.x =
                        (self.camera_offset.x + dx).rem_euclid(world.world_bounds.0);
                    self.camera_offset.y =
                        (self.camera_offset.y + dy).rem_euclid(world.world_bounds.1);
                }
            }
        }
    }

    fn draw_wrapped_circle(
        &self,
        canvas: &mut Canvas,
        ctx: &Context,
        pos: Point2<f32>,
        radius: f32,
        color: Color,
        world_bounds: (f32, f32),
    ) -> GameResult {
        let view_left = self.camera_offset.x;
        let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
        let view_top = self.camera_offset.y;
        let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;

        // 3x3グリッドでの描画位置
        let positions = [
            (pos.x, pos.y),                                   // 中央
            (pos.x - world_bounds.0, pos.y),                  // 左
            (pos.x + world_bounds.0, pos.y),                  // 右
            (pos.x, pos.y - world_bounds.1),                  // 上
            (pos.x, pos.y + world_bounds.1),                  // 下
            (pos.x - world_bounds.0, pos.y - world_bounds.1), // 左上
            (pos.x - world_bounds.0, pos.y + world_bounds.1), // 左下
            (pos.x + world_bounds.0, pos.y - world_bounds.1), // 右上
            (pos.x + world_bounds.0, pos.y + world_bounds.1), // 右下
        ];

        for &(x, y) in &positions {
            // ビューポート内にある場合のみ描画
            if x >= view_left - radius
                && x <= view_right + radius
                && y >= view_top - radius
                && y <= view_bottom + radius
            {
                let circle =
                    Mesh::new_circle(ctx, graphics::DrawMode::fill(), [x, y], radius, 0.1, color)?;
                canvas.draw(&circle, DrawParam::default());
            }
        }
        Ok(())
    }

    fn draw_wrapped_line(
        &self,
        canvas: &mut Canvas,
        ctx: &Context,
        start: Point2<f32>,
        end: Point2<f32>,
        width: f32,
        color: Color,
        world_bounds: (f32, f32),
    ) -> GameResult {
        let view_left = self.camera_offset.x;
        let view_right = self.camera_offset.x + self.window_size.0 / self.zoom;
        let view_top = self.camera_offset.y;
        let view_bottom = self.camera_offset.y + self.window_size.1 / self.zoom;

        // 3x3グリッドでの描画位置
        let offsets = [
            (0.0, 0.0),                         // 中央
            (-world_bounds.0, 0.0),             // 左
            (world_bounds.0, 0.0),              // 右
            (0.0, -world_bounds.1),             // 上
            (0.0, world_bounds.1),              // 下
            (-world_bounds.0, -world_bounds.1), // 左上
            (-world_bounds.0, world_bounds.1),  // 左下
            (world_bounds.0, -world_bounds.1),  // 右上
            (world_bounds.0, world_bounds.1),   // 右下
        ];

        for &(dx, dy) in &offsets {
            let s = Point2::new(start.x + dx, start.y + dy);
            let e = Point2::new(end.x + dx, end.y + dy);

            // ビューポート内にある場合のみ描画
            if (s.x >= view_left || e.x >= view_left)
                && (s.x <= view_right || e.x <= view_right)
                && (s.y >= view_top || e.y >= view_top)
                && (s.y <= view_bottom || e.y <= view_bottom)
            {
                let line = Mesh::new_line(ctx, &[[s.x, s.y], [e.x, e.y]], width, color)?;
                canvas.draw(&line, DrawParam::default());
            }
        }
        Ok(())
    }

    pub fn render(&mut self, ctx: &mut Context, world: &World) -> GameResult {
        self.update_camera(world);
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        // ビューポートの設定を修正
        let view_width = self.window_size.0 / self.zoom;
        let view_height = self.window_size.1 / self.zoom;
        canvas.set_screen_coordinates(graphics::Rect::new(
            self.camera_offset.x,
            self.camera_offset.y,
            view_width,
            view_height,
        ));

        // ビューポートの範囲をデバッグ表示（開発用）
        let viewport_border = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect::new(
                self.camera_offset.x,
                self.camera_offset.y,
                view_width,
                view_height,
            ),
            Color::YELLOW,
        )?;
        canvas.draw(&viewport_border, DrawParam::default());

        // Draw food sources
        for food in &world.food_manager.foods {
            self.draw_wrapped_circle(
                &mut canvas,
                ctx,
                food.position,
                food.size,
                food.color,
                world.world_bounds,
            )?;
        }

        // Draw creatures
        for (i, creature) in world.creatures.iter().enumerate() {
            // Creature body
            self.draw_wrapped_circle(
                &mut canvas,
                ctx,
                creature.physics.position,
                10.0,
                creature.color,
                world.world_bounds,
            )?;

            // Direction indicator with mode color
            let end_pos = Point2::new(
                creature.physics.position.x + 20.0 * creature.physics.rotation.cos(),
                creature.physics.position.y + 20.0 * creature.physics.rotation.sin(),
            );
            self.draw_wrapped_line(
                &mut canvas,
                ctx,
                creature.physics.position,
                end_pos,
                2.0,
                creature.mode_color,
                world.world_bounds,
            )?;

            // Highlight and show details for selected creature
            if let Some(selected_index) = self.selected_creature {
                if selected_index == i {
                    self.draw_wrapped_circle(
                        &mut canvas,
                        ctx,
                        creature.physics.position,
                        12.0,
                        Color::YELLOW,
                        world.world_bounds,
                    )?;

                    // Display detailed creature information
                    let details = format!(
                        "Energy: {:.2}\n\
                         Age: {:.2}\n\
                         Fitness: {:.2}\n\
                         State: {:?}\n\
                         Speed: {:.2}\n\
                         Position: ({:.0}, {:.0})\n\
                         Gender: {:?}",
                        creature.physics.energy,
                        creature.age,
                        creature.fitness,
                        creature.behavior_state,
                        creature.physics.velocity.norm(),
                        creature.physics.position.x,
                        creature.physics.position.y,
                        creature.gender,
                    );

                    // テキストの作成と設定を分離
                    let mut text = Text::new(details);
                    let details_text = text.set_scale(PxScale::from(24.0));
                    canvas.draw(
                        details_text,
                        DrawParam::default()
                            .color(Color::WHITE)
                            .dest([self.camera_offset.x + 10.0, self.camera_offset.y + 10.0]),
                    );

                    // 追従状態の表示（同様に分離）
                    if self.following_selected {
                        let mut text = Text::new("Following");
                        let following_text = text.set_scale(PxScale::from(24.0));
                        canvas.draw(
                            following_text,
                            DrawParam::default()
                                .color(Color::GREEN)
                                .dest([self.camera_offset.x + 10.0, self.camera_offset.y + 120.0]),
                        );
                    }
                }
            }
        }

        // ステータス情報（左上）
        let mut status = Text::new(format!(
            "Generation: {}\nPopulation: {}\nTime: {:.1}s\nFPS: {:.1}",
            world.generation,
            world.creatures.len(),
            world.elapsed_time,
            ctx.time.fps(),
        ));
        let status_text = status.set_scale(PxScale::from(28.0));
        canvas.draw(
            status_text,
            DrawParam::default().color(Color::WHITE).dest([
                self.camera_offset.x + 30.0, // Adjusted X position
                self.camera_offset.y + 50.0, // Adjusted Y position
            ]),
        );

        // 選択された生物の詳細情報（右側）
        if let Some(selected_index) = self.selected_creature {
            if let Some(creature) = world.creatures.get(selected_index) {
                let details = format!(
                    "Selected Creature\n\
                     ---------------\n\
                     Energy: {:.2}\n\
                     Age: {:.2}\n\
                     Fitness: {:.2}\n\
                     State: {:?}\n\
                     Speed: {:.2}\n\
                     Position: ({:.0}, {:.0})\n\
                     Gender: {:?}\n\
                     ---------------\n\
                     {}",
                    creature.physics.energy,
                    creature.age,
                    creature.fitness,
                    creature.behavior_state,
                    creature.physics.velocity.norm(),
                    creature.physics.position.x,
                    creature.physics.position.y,
                    creature.gender,
                    if self.following_selected {
                        "[Following]"
                    } else {
                        ""
                    }
                );

                // 半透明の背景を追加
                let text_bg = Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(
                        self.camera_offset.x + self.window_size.0 / self.zoom - 300.0,
                        self.camera_offset.y + 20.0,
                        280.0,
                        300.0,
                    ),
                    Color::new(0.0, 0.0, 0.0, 0.7),
                )?;
                canvas.draw(&text_bg, DrawParam::default());

                let mut text = Text::new(details);
                let details_text = text.set_scale(PxScale::from(24.0));
                canvas.draw(
                    details_text,
                    DrawParam::default().color(Color::WHITE).dest([
                        self.camera_offset.x + self.window_size.0 / self.zoom - 280.0,
                        self.camera_offset.y + 30.0,
                    ]),
                );
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}
