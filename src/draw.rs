use super::*;

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.camera.latitude = self.my_latitude;
        let hovered_item = self.hovered_item();
        let hovered_mailbox = self.hovered_mailbox();
        let progress = 1.0 - self.time_left / self.config.start_time;

        // Background
        ugli::clear(
            framebuffer,
            Some(Rgba::lerp(
                self.config.sky_color[0],
                self.config.sky_color[1],
                progress,
            )),
            None,
            None,
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::unit(&self.assets.sun)
                .scale_uniform(self.config.sun_size)
                .rotate(self.real_time.cos() * 0.2)
                .translate({
                    let from = vec2(-self.config.sun_offset, self.camera.fov() / 2.0 * 0.4);
                    let to = vec2(
                        self.config.sun_offset,
                        self.camera.fov() / 2.0 + self.config.sun_size,
                    );
                    from + (to - from) * progress
                }),
        );

        ugli::clear(framebuffer, None, Some(1.0), None);
        self.draw3d.draw(
            framebuffer,
            &self.camera,
            &self.road_mesh,
            ugli::DrawMode::TriangleStrip,
            &self.assets.road,
        );

        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, self.cursor);

        for item in &self.thrown_items {
            let t = item.t / self.config.throw_time;
            let up = -vec3::cross(item.to - item.from, vec3(1.0, 0.0, 0.0)).normalize_or_zero();
            let pos = item.from
                + (item.to - item.from) * t
                + up * (1.0 - (1.0 - t * 2.0).sqr()) * self.config.throw_height;
            let matrix = mat4::translate(pos)
                * mat4::rotate_x(-self.camera.latitude - self.camera.rot)
                * mat4::rotate_z(item.rot)
                * mat4::scale(item.half_size.extend(1.0) * self.config.item_throw_scale)
                * mat4::translate(vec3(-1.0, -1.0, 0.0))
                * mat4::scale_uniform(2.0);
            self.draw3d.draw_sprite_with_transform(
                framebuffer,
                &self.camera,
                &item.texture,
                matrix,
                item.color,
            );
        }

        for mailbox in &self.mailboxes {
            self.draw3d.draw_sprite(
                framebuffer,
                &self.camera,
                &self.assets.mailbox,
                self.mailbox_pos(mailbox),
                vec2::splat(self.config.mailbox_size) * vec2(-mailbox.x.signum(), 1.0),
                self.config.mailbox_colors[mailbox.color],
            );
        }

        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::new(self.bag_position, &self.assets.bag),
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::unit(&self.assets.bike)
                .translate(vec2(0.0, 1.0))
                .scale_uniform(0.5)
                .scale(self.bag_position.size() * vec2(2.0, 1.0))
                .rotate(self.real_time.sin() * 0.1)
                .translate(vec2(self.bag_position.center().x, self.bag_position.min.y)),
        );
        if let Some(item) = &self.holding {
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit_colored(&*item.texture, item.color)
                    .scale(item.half_size * self.config.item_hold_scale)
                    .rotate(item.rot)
                    .translate(cursor_world),
            );
        }
        for item in &self.juggling_items {
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit_colored(&*item.texture, item.color)
                    .scale(item.half_size)
                    .rotate(item.rot)
                    .translate(item.pos),
            );
        }
        if let Some(index) = hovered_item {
            let item = &self.juggling_items[index];
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit(&self.assets.envelope_highlight)
                    .scale(item.half_size * 1.1)
                    .rotate(item.rot)
                    .translate(item.pos),
            );
        }
        self.geng.draw2d().draw2d(
            framebuffer,
            self.camera.as_2d(),
            &draw2d::TexturedQuad::unit_colored(
                if self.holding.is_some() {
                    &self.assets.holding_hand
                } else {
                    &self.assets.hand
                },
                if self.error_animation_time < 1.0 {
                    Rgba::RED
                } else {
                    Rgba::WHITE
                },
            )
            .translate(vec2(
                (1.0 - (self.error_animation_time * 2.0 - 1.0).sqr())
                    * self.config.error_animation_distance
                    * (self.real_time * self.config.error_animation_freq).sin(),
                0.0,
            ))
            .rotate(
                (1.0 - self.cursor.x / self.framebuffer_size.x * 2.0)
                    * self.config.hand_rotation.to_radians(),
            )
            .scale_uniform(self.config.hand_radius)
            .translate(
                cursor_world
                    + (vec2(0.0, self.config.throw_target_height) - cursor_world)
                        * (1.0 - (self.throw_animation_time * 2.0 - 1.0).sqr())
                        * self.config.throw_hand_distance,
            ),
        );

        if let Some(index) = hovered_mailbox {
            let mailbox = &self.mailboxes[index];
            let camera_up = vec3::cross(self.camera.dir(), vec3(1.0, 0.0, 0.0)).normalize_or_zero();
            let pos = self.mailbox_pos(mailbox) + camera_up * self.config.mailbox_size * 0.75;
            if let Some(pos) = self.camera.world_to_screen(self.framebuffer_size, pos) {
                let pos = self
                    .camera
                    .as_2d()
                    .screen_to_world(self.framebuffer_size, pos);
                self.geng.draw2d().draw2d(
                    framebuffer,
                    self.camera.as_2d(),
                    &draw2d::TexturedQuad::unit(&self.assets.aim)
                        .scale_uniform(1.0)
                        .rotate(self.real_time)
                        .translate(pos),
                );
            }
        }

        self.draw_particles(framebuffer);

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
