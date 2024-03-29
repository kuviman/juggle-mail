use crate::final_screen::FinalScreen;

use super::*;

impl Game {
    pub fn update_impl(&mut self, delta_time: f32) {
        self.real_time += delta_time;

        self.last_score_t += delta_time;

        if self.time_left < 0.0 || self.lives == 0 {
            if self.end_timer == 0.0 && self.lives != 0 {
                self.assets.sfx.timer.play();
            }
            if self.end_timer == 0.0 {
                while let Some(touch) = self.touches.pop() {
                    self.touch_end(touch.id, touch.position);
                }
            }
            self.end_timer += delta_time / 3.0;
            if self.end_timer > 1.0 {
                self.transition = Some(geng::state::Transition::Switch(Box::new(
                    geng::LoadingScreen::new(
                        &self.geng,
                        geng::EmptyLoadingScreen::new(&self.geng),
                        {
                            let geng = self.geng.clone();
                            let assets = self.assets.clone();
                            let config = self.config.clone();
                            let diff = self.diff.clone();
                            let score = self.score;
                            let name = self.name.clone();
                            async move {
                                #[cfg(feature = "leaderboard")]
                                let pos_and_scores =
                                    Some(leaderboard::submit(diff.clone(), &name, score).await);
                                #[cfg(not(feature = "leaderboard"))]
                                let pos_and_scores = None;
                                FinalScreen::new(
                                    &geng,
                                    &assets,
                                    &config,
                                    diff,
                                    score,
                                    name,
                                    pos_and_scores,
                                )
                            }
                        },
                    ),
                )));
            }
        }

        self.add_raw_score(delta_time * self.config.juggling_score_multiplier);
        self.time_left -= delta_time;

        for touch in &mut self.touches {
            if let Some(remove_time) = &mut touch.remove_time {
                *remove_time =
                    (*remove_time + delta_time / self.config.throw_animation_time).min(1.0);
            }
            touch.throw_animation_time = (touch.throw_animation_time
                + delta_time / self.config.throw_animation_time)
                .min(1.0);
            if touch.holding.is_some() {
                touch.throw_animation_time = 0.0;
            }
            touch.error_animation_time = (touch.error_animation_time
                + delta_time / self.config.error_animation_time)
                .min(1.0);
            if touch.holding.is_some() {
                touch.error_animation_time = 1.0;
            }
        }
        self.touches.retain(|touch| {
            touch.id == None
                || match touch.remove_time {
                    Some(time) => time < 1.0,
                    None => true,
                }
        });

        self.update_particles(delta_time);

        let delta_time = delta_time * self.diff.time_scale;

        self.update_juggling_items(delta_time);
        self.my_latitude += self.config.ride_speed * delta_time; // Move forward
        self.update_mailboxes();
        self.update_houses();
        self.update_thrown_items(delta_time);
    }

    fn add_raw_score(&mut self, raw_score: f32) {
        if self.end_timer != 0.0 {
            return;
        }
        let multiplier = self.juggling_items.len()
            + 1
            + self
                .touches
                .iter()
                .filter(|touch| touch.holding.is_some())
                .count();
        let scored = raw_score * multiplier as f32;
        self.score += scored;
        if raw_score == self.config.deliver_score {
            self.last_score_t = 0.0;
            self.last_score_text = format!("+{}", scored.floor() as i32);
        }
    }

    fn update_juggling_items(&mut self, delta_time: f32) {
        for item in &mut self.juggling_items {
            item.vel.y -= self.config.gravity * delta_time;
            item.pos += item.vel * delta_time;
            item.rot += item.w * delta_time;
        }
        let mut lives_lost = 0;
        let mut spawn_particles = None;
        self.juggling_items.retain(|item| {
            if item.pos.y > self.bag_position.min.y {
                true
            } else {
                lives_lost += 1;
                spawn_particles = Some((item.pos, self.config.explosion_color));
                false
            }
        });
        if let Some((pos, color)) = spawn_particles {
            self.particles_ui
                .extend(self.spawn_particles(pos.extend(0.0), color));
        }
        for _ in 0..lives_lost {
            self.lose_life();
        }
    }

    fn update_houses(&mut self) {
        self.houses.retain(|house| {
            house.latitude > self.my_latitude - self.config.despawn_distance.to_radians()
        });
        while self.houses.last().map_or(true, |house| {
            house.latitude < self.my_latitude + self.config.spawn_distance.to_radians()
        }) {
            let last_latitude = self
                .houses
                .last()
                .map_or(self.my_latitude, |house| house.latitude);
            for x in [-1, 1] {
                self.houses.push(House {
                    x: x as f32 * (self.config.road_width + self.config.house_offset),
                    latitude: last_latitude + self.config.distance_between_houses.to_radians(),
                    texture: thread_rng().gen_range(0..self.assets.houses.len()),
                });
            }
        }
    }

    fn update_mailboxes(&mut self) {
        self.mailboxes.retain(|mailbox| {
            mailbox.latitude > self.my_latitude - self.config.despawn_distance.to_radians()
        });
        while self.mailboxes.last().map_or(true, |mailbox| {
            mailbox.latitude < self.my_latitude + self.config.spawn_distance.to_radians()
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
                    color: thread_rng().gen_range(0..self.config.mailbox_colors.len()),
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
        let mut raw_score_added = 0.0;
        let mut lives_lost = 0;
        let mut spawn_particles = None;
        self.thrown_items.retain(|item| {
            if item.t < self.config.throw_time {
                true
            } else {
                let index = self
                    .mailboxes
                    .iter()
                    .position(|mailbox| mailbox.id == item.to_id);
                if let Some(index) = index {
                    raw_score_added += self.config.deliver_score;
                    self.assets.sfx.score.play_random_pitch();
                    self.mailboxes.remove(index);
                    spawn_particles = Some((item.to, self.config.score_color));
                } else {
                    spawn_particles = Some((item.to, self.config.explosion_color));
                    lives_lost += 1;
                }
                false
            }
        });
        if let Some((pos, color)) = spawn_particles {
            self.particles_3d.extend(self.spawn_particles(pos, color));
        }
        for _ in 0..lives_lost {
            self.lose_life();
        }
        self.add_raw_score(raw_score_added);
    }

    fn lose_life(&mut self) {
        if self.lives != 0 {
            self.lives -= 1;
            self.assets.sfx.explosion.play_random_pitch();
            if self.lives == 0 {
                self.lose_sfx = Some(self.assets.sfx.lose.play());
                self.music.stop();
            }
        }
    }
}
