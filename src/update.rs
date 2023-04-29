use super::*;

impl Game {
    pub fn update_impl(&mut self, delta_time: f32) {
        if self.time_left < 0.0 || self.lives == 0 {
            // TODO
            return;
        }

        self.score +=
            delta_time * self.juggling_items.len() as f32 * self.config.juggling_score_multiplier;
        self.time_left -= delta_time;

        let delta_time = delta_time * self.config.time_scale;

        self.update_juggling_items(delta_time);
        self.my_latitude += self.config.ride_speed * delta_time; // Move forward
        self.update_mailboxes();
        self.update_thrown_items(delta_time);
    }

    fn update_juggling_items(&mut self, delta_time: f32) {
        for item in &mut self.juggling_items {
            item.vel.y -= self.config.gravity * delta_time;
            item.pos += item.vel * delta_time;
            item.rot += item.w * delta_time;
        }
        self.juggling_items.retain(|item| {
            if item.pos.y > self.bag_position.min.y {
                true
            } else {
                if self.lives != 0 {
                    self.lives -= 1;
                }
                false
            }
        });
    }

    fn update_mailboxes(&mut self) {
        self.mailboxes
            .retain(|mailbox| mailbox.latitude > self.my_latitude - f32::PI);
        while self.mailboxes.last().map_or(true, |mailbox| {
            mailbox.latitude < self.my_latitude + f32::PI
        }) {
            let last_latitude = self
                .mailboxes
                .last()
                .map_or(self.my_latitude, |mailbox| mailbox.latitude);
            let (left, right) = if thread_rng().gen_bool(self.config.double_mailbox_probability) {
                (true, true)
            } else if thread_rng().gen() {
                (true, false)
            } else {
                (false, true)
            };
            for (x, spawn) in itertools::izip![[-1, 1], [left, right]] {
                if !spawn {
                    continue;
                }
                self.mailboxes.push(Mailbox {
                    id: self.next_id,
                    x: x as f32 * (self.config.road_width + self.config.mailbox_size / 2.0),
                    latitude: last_latitude + self.config.distance_between_mailboxes.to_radians(),
                    color: thread_rng().gen_range(0..self.config.colors.len()),
                });
                self.next_id += 1;
            }
        }
    }

    fn update_thrown_items(&mut self, delta_time: f32) {
        for item in &mut self.thrown_items {
            item.t += delta_time;
            item.item.rot += item.item.w * delta_time;
        }
        self.thrown_items.retain(|item| {
            if item.t < self.config.throw_time {
                true
            } else {
                let index = self
                    .mailboxes
                    .iter()
                    .position(|mailbox| mailbox.id == item.to_id);
                if let Some(index) = index {
                    let mailbox = &self.mailboxes[index];
                    if mailbox.color == item.color {
                        self.score += self.config.deliver_score;
                        self.mailboxes.remove(index);
                    }
                } else if self.lives != 0 {
                    self.lives -= 1;
                }
                false
            }
        });
    }
}
