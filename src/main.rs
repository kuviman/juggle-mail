use geng::prelude::*;

mod camera;
mod draw3d;

use camera::*;
use draw3d::Draw3d;

type Id = usize;

#[derive(Deserialize)]
pub struct Config {
    pub sky_color: Rgba<f32>,
    pub gravity: f32,
    pub throw_speed: f32,
    pub throw_angle: f32,
    pub item_scale: f32,
    pub item_hold_scale: f32,
    pub hand_radius: f32,
    pub item_max_w: f32,
    pub throw_target_height: f32,
    pub ui_fov: f32,
    pub fov: f32,
    pub earth_radius: f32,
    pub ride_speed: f32,
    pub camera_height: f32,
    pub camera_rot: f32,
    pub road_width: f32,
    pub mailbox_size: f32,
    pub distance_between_mailboxes: f32,
    pub throw_time: f32,
    pub item_throw_max_w: f32,
    pub item_throw_scale: f32,
    pub throw_height: f32,
    pub colors: Vec<Rgba<f32>>,
    pub double_mailbox_probability: f64,
    pub time_scale: f32,
    pub start_time: f32,
    pub lives: usize,
    pub juggling_score_multiplier: f32,
    pub deliver_score: f32,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub sprite: ugli::Program,
    pub mesh3d: ugli::Program,
}

#[derive(Deref, DerefMut)]
struct Texture(#[deref] ugli::Texture);

impl std::borrow::Borrow<ugli::Texture> for &Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}

impl geng::asset::Load for Texture {
    fn load(manager: &geng::Manager, path: &std::path::Path) -> geng::asset::Future<Self> {
        let texture = manager.load(path);
        async move {
            let mut texture: ugli::Texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Self(texture))
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = ugli::Texture::DEFAULT_EXT;
}

#[derive(geng::asset::Load)]
pub struct Assets {
    shaders: Shaders,
    envelope: Rc<Texture>,
    bag: Texture,
    hand: Texture,
    holding_hand: Texture,
    mailbox: Texture,
    #[load(postprocess = "make_repeated")]
    road: Texture,
}

fn make_repeated(texture: &mut Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
}

struct Item {
    texture: Rc<Texture>,
    pos: vec2<f32>,
    vel: vec2<f32>,
    rot: f32,
    w: f32,
    half_size: vec2<f32>,
    color: usize,
}

impl Item {
    pub fn new(texture: &Rc<Texture>, scale: f32, color: usize) -> Self {
        Self {
            texture: texture.clone(),
            pos: vec2::ZERO,
            vel: vec2::ZERO,
            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
            w: 0.0,
            half_size: vec2(texture.size().map(|x| x as f32).aspect(), 1.0) * scale,
            color,
        }
    }
}

#[derive(Deref)]
struct ThrownItem {
    #[deref]
    pub item: Item,
    pub from: vec3<f32>,
    pub to: vec3<f32>,
    pub t: f32,
    pub to_id: Id,
}

struct Mailbox {
    pub id: Id,
    pub x: f32,
    pub latitude: f32,
    pub color: usize,
}

struct Game {
    score: f32,
    time_left: f32,
    next_id: Id,
    framebuffer_size: vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    config: Rc<Config>,
    camera: Camera,
    juggling_items: Vec<Item>,
    thrown_items: Vec<ThrownItem>,
    bag_position: Aabb2<f32>,
    holding: Option<Item>,
    mailboxes: Vec<Mailbox>,
    draw3d: Draw3d,
    my_latitude: f32,
    road_mesh: ugli::VertexBuffer<draw3d::Vertex>,
    transition: Option<geng::state::Transition>,
    lives: usize,
}

impl Game {
    pub fn mailbox_pos(&self, mailbox: &Mailbox) -> vec3<f32> {
        let circle_pos = vec2(self.config.earth_radius, 0.0).rotate(mailbox.latitude);
        vec3(mailbox.x, circle_pos.x, -circle_pos.y)
    }
    pub fn hovered_mailbox(&self) -> Option<usize> {
        // self.hovered_mailbox();
        let ray = self.camera.pixel_ray(
            self.framebuffer_size,
            self.geng.window().cursor_position().map(|x| x as f32),
        );
        let camera_dir = self.camera.dir();
        let right = vec3(1.0, 0.0, 0.0);
        let up = vec3::cross(camera_dir, right).normalize_or_zero();
        self.mailboxes.iter().position(|mailbox| {
            let pos = self.mailbox_pos(mailbox);
            // dot(ray.from + ray.dir * t - pos, camera_dir) = 0
            let t = vec3::dot(pos - ray.from, camera_dir) / vec3::dot(ray.dir, camera_dir);
            if t < 0.0 {
                return false;
            }
            let p = ray.from + ray.dir * t;

            let p = vec2(vec3::dot(p - pos, right), vec3::dot(p - pos, up));
            p.x.abs() < self.config.mailbox_size / 2.0
                && p.y > 0.0
                && p.y < self.config.mailbox_size
        })
    }
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Rc<Config>) -> Self {
        let camera = Camera::new(
            config.fov.to_radians(),
            config.ui_fov,
            config.camera_rot.to_radians(),
            config.earth_radius + config.camera_height,
        );
        Self {
            lives: config.lives,
            score: 0.0,
            time_left: config.start_time,
            transition: None,
            next_id: 0,
            framebuffer_size: vec2::splat(1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            bag_position: Aabb2::point(vec2(0.0, -camera.fov() / 2.0 + 1.0)).extend_uniform(1.0),
            camera,
            juggling_items: vec![],
            holding: None,
            mailboxes: vec![],
            draw3d: Draw3d::new(geng, assets),
            my_latitude: 0.0,
            road_mesh: ugli::VertexBuffer::new_static(geng.ugli(), {
                const N: usize = 100;
                (0..=N)
                    .flat_map(|i| {
                        let yz = vec2(config.earth_radius, 0.0)
                            .rotate(2.0 * f32::PI * i as f32 / N as f32);
                        let uv_y =
                            (2.0 * f32::PI * config.earth_radius).ceil() * i as f32 / N as f32;
                        [-1, 1].map(|x| draw3d::Vertex {
                            a_pos: vec3(x as f32 * config.road_width, yz.x, yz.y),
                            a_uv: vec2(x as f32 * 0.5 + 0.5, uv_y),
                        })
                    })
                    .collect()
            }),
            thrown_items: vec![],
        }
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                let pos = self
                    .camera
                    .as_2d()
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if let Some(index) = self.juggling_items.iter().rposition(|item| {
                    Aabb2::ZERO.extend_uniform(1.0).contains(
                        (Quad::unit()
                            .scale(item.half_size.map(|x| x + self.config.hand_radius))
                            .rotate(item.rot)
                            .translate(item.pos)
                            .transform
                            .inverse()
                            * pos.extend(1.0))
                        .into_2d(),
                    )
                }) {
                    self.holding = Some(self.juggling_items.remove(index));
                } else if self
                    .bag_position
                    .extend_uniform(self.config.hand_radius)
                    .contains(pos)
                {
                    self.holding = Some(Item::new(
                        &self.assets.envelope,
                        self.config.item_scale,
                        thread_rng().gen_range(0..self.config.colors.len()),
                    ));
                }
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let pos = self
                    .camera
                    .as_2d()
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if let Some(mut item) = self.holding.take() {
                    if let Some(index) = self.hovered_mailbox() {
                        let mailbox = &self.mailboxes[index];
                        item.w = thread_rng().gen_range(-1.0..1.0) * self.config.item_throw_max_w;
                        // Shoutout to Foggy's mom
                        let mut pixel_ray = self
                            .camera
                            .pixel_ray(self.framebuffer_size, position.map(|x| x as f32));
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
                    } else {
                        item.pos = pos;
                        item.vel = (vec2(0.0, self.config.throw_target_height) - item.pos)
                            .normalize_or_zero()
                            .rotate(thread_rng().gen_range(
                                -self.config.throw_angle.to_radians()
                                    ..self.config.throw_angle.to_radians(),
                            ))
                            * self.config.throw_speed;
                        item.w = thread_rng().gen_range(-1.0..1.0) * self.config.item_max_w;
                        self.juggling_items.push(item);
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.transition = Some(geng::state::Transition::Switch(Box::new(Game::new(
                    &self.geng,
                    &self.assets,
                    &self.config,
                ))));
            }
            _ => {}
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        if self.time_left < 0.0 || self.lives == 0 {
            // TODO
            return;
        }

        self.score +=
            delta_time * self.juggling_items.len() as f32 * self.config.juggling_score_multiplier;
        self.time_left -= delta_time;

        let delta_time = delta_time * self.config.time_scale;

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

        self.my_latitude += self.config.ride_speed * delta_time;

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
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
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
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
}

fn main() {
    let geng = Geng::new("Ludum53");
    geng.clone().run_loading(async move {
        let assets: Rc<Assets> = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let config: Config = file::load_detect(run_dir().join("assets").join("config.toml"))
            .await
            .unwrap();
        let config = Rc::new(config);
        Game::new(&geng, &assets, &config)
    })
}
