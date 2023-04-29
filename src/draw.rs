use super::*;

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.camera.latitude = self.my_latitude;
        let hovered_mailbox = self.hovered_mailbox();
        ugli::clear(framebuffer, Some(self.config.sky_color), Some(1.0), None);
        self.draw3d.draw(
            framebuffer,
            &self.camera,
            &self.road_mesh,
            ugli::DrawMode::TriangleStrip,
            &self.assets.road,
        );

        let mouse_pos = self.camera.as_2d().screen_to_world(
            self.framebuffer_size,
            self.geng.window().cursor_position().map(|x| x as f32),
        );

        for item in &self.thrown_items {
            let t = item.t / self.config.throw_time;
            let up = -vec3::cross(item.to - item.from, vec3(1.0, 0.0, 0.0)).normalize_or_zero();
            let pos = item.from
                + (item.to - item.from) * t
                + up * (1.0 - (1.0 - t * 2.0).sqr()) * self.config.throw_height;
            self.draw3d.draw_sprite_with_transform(
                framebuffer,
                &self.camera,
                &item.texture,
                mat4::translate(pos)
                    * mat4::rotate_x(-self.camera.latitude - self.camera.rot)
                    * mat4::rotate_z(item.rot)
                    * mat4::scale(item.half_size.extend(1.0) * self.config.item_throw_scale)
                    * mat4::translate(vec3(-1.0, -1.0, 0.0))
                    * mat4::scale_uniform(2.0),
                self.config.colors[item.color],
            );
        }

        for (index, mailbox) in self.mailboxes.iter().enumerate() {
            self.draw3d.draw_sprite(
                framebuffer,
                &self.camera,
                &self.assets.mailbox,
                self.mailbox_pos(mailbox),
                vec2::splat(self.config.mailbox_size),
                self.config.colors[mailbox.color],
            );
            if Some(index) == hovered_mailbox {
                // TODO
            }
        }

        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::new(self.bag_position, &self.assets.bag),
        );
        if let Some(item) = &self.holding {
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit_colored(&*item.texture, self.config.colors[item.color])
                    .scale(item.half_size * self.config.item_hold_scale)
                    .rotate(item.rot)
                    .translate(mouse_pos),
            );
        }
        for item in &self.juggling_items {
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit_colored(&*item.texture, self.config.colors[item.color])
                    .scale(item.half_size)
                    .rotate(item.rot)
                    .translate(item.pos),
            );
        }

        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::unit(if self.holding.is_some() {
                &self.assets.holding_hand
            } else {
                &self.assets.hand
            })
            .scale_uniform(self.config.hand_radius)
            .translate(mouse_pos),
        );

        self.geng.default_font().draw(
            framebuffer,
            self.camera.as_2d(),
            &format!(
                "score: {}\ntime left: {:.3} secs",
                self.score.floor() as i32,
                self.time_left
            ),
            vec2::splat(geng::TextAlign::CENTER),
            mat3::translate(vec2(0.0, 4.0)) * mat3::scale_uniform(0.5),
            Rgba::BLACK,
        );

        self.geng.default_font().draw(
            framebuffer,
            self.camera.as_2d(),
            &format!("lives {}", self.lives),
            vec2::splat(geng::TextAlign::CENTER),
            mat3::translate(vec2(0.0, 4.5)) * mat3::scale_uniform(0.5),
            Rgba::BLACK,
        );
    }
}
