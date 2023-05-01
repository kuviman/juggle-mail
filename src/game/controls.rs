use super::*;

impl Game {
    pub fn hovered_item(&self) -> Option<usize> {
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, self.cursor);
        self.juggling_items
            .iter()
            .enumerate()
            .filter(|(_index, item)| {
                Aabb2::ZERO.extend_uniform(1.0).contains(
                    (Quad::unit()
                        .scale(item.half_size.map(|x| x + self.config.hand_radius))
                        .rotate(item.rot)
                        .translate(item.pos)
                        .transform
                        .inverse()
                        * cursor_world.extend(1.0))
                    .into_2d(),
                )
            })
            .min_by_key(|(_index, item)| r32((item.pos - cursor_world).len()))
            .map(|(index, _item)| index)
    }

    pub fn start_drag(&mut self) {
        if self.end_timer != 0.0 {
            return;
        }
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, self.cursor);
        if let Some(index) = self.hovered_item() {
            self.assets.sfx.pick.play_random_pitch();
            self.holding = Some(self.juggling_items.remove(index));
        } else if self
            .bag_position
            .extend_uniform(self.config.hand_radius)
            .contains(cursor_world)
        {
            self.assets.sfx.pick.play_random_pitch();
            self.holding = Some(Item::new(&self.assets.envelope, self.config.item_scale));
        } else {
            self.error_animation_time = 0.0;
            self.assets.sfx.error.play_random_pitch();
        }
    }

    pub fn end_drag(&mut self) {
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, self.cursor);
        if let Some(mut item) = self.holding.take() {
            self.throw_animation_time = 0.0;
            if let Some(index) = self.hovered_mailbox() {
                let mailbox = &self.mailboxes[index];
                item.w = self.config.item_throw_max_w * mailbox.x.signum();
                // Shoutout to Foggy's mom
                let mut pixel_ray = self.camera.pixel_ray(self.framebuffer_size, self.cursor);
                let cam_dir = self.camera.dir();
                pixel_ray.dir -= cam_dir * vec3::dot(cam_dir, pixel_ray.dir);
                pixel_ray.dir += cam_dir;
                let item = ThrownItem {
                    item,
                    from: pixel_ray.from + pixel_ray.dir.normalize_or_zero(),
                    to: self.mailbox_pos(mailbox).normalize_or_zero()
                        * (self.config.earth_radius + self.config.mailbox_size),
                    t: 0.0,
                    to_id: mailbox.id,
                };
                self.thrown_items.push(item);
                self.assets.sfx.throw.play_random_pitch();
            } else {
                item.pos = cursor_world;
                item.vel = (vec2(0.0, self.config.throw_target_height) - item.pos).rotate(
                    thread_rng().gen_range(
                        -self.config.throw_angle.to_radians()..self.config.throw_angle.to_radians(),
                    ),
                ) * self.config.throw_speed
                    / self.config.throw_target_height;
                item.w = thread_rng().gen_range(-1.0..1.0) * self.config.item_max_w;
                self.juggling_items.push(item);
                self.assets.sfx.juggle.play_random_pitch();
            }
        }
    }
}