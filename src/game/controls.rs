use super::*;

impl Game {
    pub fn hovered_item(&self, cursor: vec2<f32>) -> Option<usize> {
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, cursor);
        self.juggling_items
            .iter()
            .enumerate()
            .filter(|(_index, item)| {
                Aabb2::ZERO.extend_uniform(1.0).contains(
                    (Quad::unit()
                        .scale(item.half_size.map(|x| x + self.config.hand_radius))
                        .rotate(Angle::from_radians(item.rot))
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

    pub fn touch_start(&mut self, id: Option<u64>, position: vec2<f32>) {
        if self.end_timer != 0.0 {
            return;
        }
        if self
            .touches
            .iter()
            .any(|touch| touch.id == id && touch.holding.is_some())
        {
            return;
        }
        self.touches.retain(|touch| touch.id != id);
        let mut touch = Touch {
            id,
            position,
            holding: None,
            error_animation_time: 1.0,
            throw_animation_time: 1.0,
            remove_time: None,
        };
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, touch.position.map(|x| x as f32));
        if let Some(index) = self.hovered_item(touch.position) {
            self.assets.sfx.pick.play_random_pitch();
            touch.holding = Some(self.juggling_items.remove(index));
        } else if self
            .bag_position
            .extend_uniform(self.config.hand_radius)
            .contains(cursor_world)
        {
            self.assets.sfx.pick.play_random_pitch();

            let skin_assets = self
                .assets
                .skins
                .get(&self.name)
                .unwrap_or(&self.assets.skins["default"]);
            touch.holding = Some(Item::new(&skin_assets.newspaper, self.config.item_scale));
        } else {
            touch.error_animation_time = 0.0;
            self.assets.sfx.error.play_random_pitch();
        }
        self.touches.push(touch);
    }

    pub fn touch_move(&mut self, id: Option<u64>, position: vec2<f32>) {
        if let Some(touch) = self.touches.iter_mut().find(|touch| touch.id == id) {
            touch.position = position;
        } else {
            self.touches.push(Touch {
                id,
                position,
                holding: None,
                error_animation_time: 1.0,
                throw_animation_time: 1.0,
                remove_time: None,
            });
        }
    }

    pub fn touch_end(&mut self, id: Option<u64>, position: vec2<f32>) {
        let Some(touch_index) = self.touches.iter_mut().position(|touch| touch.id == id) else {
            return;
        };
        let mut touch = self.touches.remove(touch_index);
        touch.position = position;
        let cursor_world = self
            .camera
            .as_2d()
            .screen_to_world(self.framebuffer_size, touch.position.map(|x| x as f32));
        if let Some(mut item) = touch.holding.take() {
            touch.throw_animation_time = 0.0;
            if let Some(index) = self.hovered_mailbox(touch.position) {
                let mailbox = &self.mailboxes[index];
                item.w = self.config.item_throw_max_w * mailbox.x.signum();
                // Shoutout to Foggy's mom
                let mut pixel_ray = self
                    .camera
                    .pixel_ray(self.framebuffer_size, touch.position.map(|x| x as f32));
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
                    Angle::from_radians(thread_rng().gen_range(
                        -self.config.throw_angle.to_radians()..self.config.throw_angle.to_radians(),
                    )),
                ) * self.config.throw_speed
                    / self.config.throw_target_height;
                item.w = thread_rng().gen_range(-1.0..1.0) * self.config.item_max_w;
                self.juggling_items.push(item);
                self.assets.sfx.juggle.play_random_pitch();
            }
        }
        touch.remove_time = Some(0.0);
        self.touches.push(touch);
    }
}
