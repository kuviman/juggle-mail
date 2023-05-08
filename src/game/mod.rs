use super::*;

type Id = usize;

mod controls;
mod draw;
mod particle;
mod update;

use particle::*;

struct Item {
    texture: Rc<Texture>,
    pos: vec2<f32>,
    vel: vec2<f32>,
    rot: f32,
    w: f32,
    half_size: vec2<f32>,
    color: Rgba<f32>,
}

impl Item {
    pub fn new(texture: &Rc<Texture>, scale: f32) -> Self {
        Self {
            texture: texture.clone(),
            pos: vec2::ZERO,
            vel: vec2::ZERO,
            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
            w: 0.0,
            half_size: vec2(texture.size().map(|x| x as f32).aspect(), 1.0) * scale,
            color: {
                Rgba::new(
                    thread_rng().gen_range(0.9..1.0),
                    thread_rng().gen_range(0.9..1.0),
                    thread_rng().gen_range(0.9..1.0),
                    1.0,
                )
            },
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

struct House {
    pub x: f32,
    pub latitude: f32,
    pub texture: usize,
}

struct Touch {
    id: Option<u64>,
    position: vec2<f32>,
    holding: Option<Item>,
    error_animation_time: f32,
    throw_animation_time: f32,
    remove_time: Option<f32>,
}

pub struct Game {
    diff: Difficulty,
    real_time: f32,
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
    mailboxes: Vec<Mailbox>,
    houses: Vec<House>,
    draw3d: Draw3d,
    my_latitude: f32,
    road_mesh: ugli::VertexBuffer<draw3d::Vertex>,
    transition: Option<geng::state::Transition>,
    lives: usize,
    touches: Vec<Touch>,
    music: geng::SoundEffect,
    particles_ui: Vec<Particle>,
    particles_3d: Vec<Particle>,
    last_score_text: String,
    last_score_t: f32,
    end_timer: f32,
    lose_sfx: Option<geng::SoundEffect>,
}

impl Drop for Game {
    fn drop(&mut self) {
        self.music.stop();
        if let Some(mut sfx) = self.lose_sfx.take() {
            sfx.stop();
        }
    }
}

impl Game {
    fn mailbox_pos(&self, mailbox: &Mailbox) -> vec3<f32> {
        let circle_pos = vec2(self.config.earth_radius, 0.0).rotate(mailbox.latitude);
        vec3(mailbox.x, circle_pos.x, -circle_pos.y)
    }
    fn hovered_mailbox(&self, cursor: vec2<f32>) -> Option<usize> {
        // self.hovered_mailbox();
        let ray = self.camera.pixel_ray(self.framebuffer_size, cursor);
        let camera_dir = self.camera.dir();
        let right = vec3(1.0, 0.0, 0.0);
        let up = vec3::cross(camera_dir, right).normalize_or_zero();
        self.mailboxes.iter().position(|mailbox| {
            let pos = self.mailbox_pos(mailbox);
            // dot(ray.from + ray.dir * t - pos, camera_dir) = 0
            let t = vec3::dot(pos - ray.from, camera_dir) / vec3::dot(ray.dir, camera_dir);
            if t < 0.0 || t * ray.dir.len() > self.config.max_throw_distance {
                return false;
            }
            let p = ray.from + ray.dir * t;

            let p = vec2(vec3::dot(p - pos, right), vec3::dot(p - pos, up));
            p.x.abs() < self.config.mailbox_size / 2.0
                && p.y > 0.0
                && p.y < self.config.mailbox_size
        })
    }
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Rc<Config>, diff: Difficulty) -> Self {
        let camera = Camera::new(
            config.fov.to_radians(),
            config.ui_fov,
            config.camera_rot.to_radians(),
            config.earth_radius + config.camera_height,
        );
        let mut music = assets.music.play();
        music.set_volume(0.4);
        Self {
            lose_sfx: None,
            end_timer: 0.0,
            diff: diff.clone(),
            houses: vec![],
            real_time: 0.0,
            music,
            lives: diff.lives,
            score: 0.0,
            time_left: diff.game_time,
            transition: None,
            next_id: 0,
            framebuffer_size: vec2::splat(1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            bag_position: Aabb2::point(vec2(0.0, -camera.fov() / 2.0 + 1.0)).extend_uniform(1.0),
            camera,
            juggling_items: vec![],
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
                            a_pos: vec3(x as f32 * 100.0, yz.x, yz.y),
                            a_uv: vec2(x as f32 * 20.0 + 0.5, uv_y),
                        })
                    })
                    .collect()
            }),
            thrown_items: vec![],
            touches: vec![],
            particles_3d: vec![],
            particles_ui: vec![],
            last_score_t: 1.0,
            last_score_text: "".to_owned(),
        }
    }

    fn restart(&mut self) {
        self.transition = Some(geng::state::Transition::Switch(Box::new(Game::new(
            &self.geng,
            &self.assets,
            &self.config,
            self.diff.clone(),
        ))));
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, .. } => {
                self.touch_start(None, position.map(|x| x as f32));
            }
            geng::Event::MouseMove { position, .. } => {
                self.touch_move(None, position.map(|x| x as f32));
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                self.touch_end(None, position.map(|x| x as f32));
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.restart();
            }
            geng::Event::KeyDown {
                key: geng::Key::Escape | geng::Key::Backspace | geng::Key::Enter,
            } => {
                self.transition = Some(geng::state::Transition::Pop);
            }
            geng::Event::KeyDown { .. } => {
                self.touch_start(None, self.geng.window().cursor_position().map(|x| x as f32));
            }
            geng::Event::KeyUp { .. } => {
                self.touch_end(None, self.geng.window().cursor_position().map(|x| x as f32));
            }
            geng::Event::TouchStart(touch) => {
                self.touch_start(Some(touch.id), touch.position.map(|x| x as f32));
            }
            geng::Event::TouchMove(touch) => {
                self.touch_move(Some(touch.id), touch.position.map(|x| x as f32));
            }
            geng::Event::TouchEnd(touch) => {
                self.touch_end(Some(touch.id), touch.position.map(|x| x as f32));
            }
            _ => {}
        }
    }
    fn update(&mut self, delta_time: f64) {
        self.geng.window().set_cursor_type(geng::CursorType::None);
        self.update_impl(delta_time as f32);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        self.draw_impl(framebuffer);
    }
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
}
