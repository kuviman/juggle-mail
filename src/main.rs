use geng::prelude::*;

mod assets;
mod camera;
mod config;
mod controls;
mod draw;
mod draw3d;
mod update;

use assets::*;
use camera::*;
use config::*;
use draw3d::Draw3d;

type Id = usize;

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
    cursor: vec2<f32>,
}

impl Game {
    pub fn mailbox_pos(&self, mailbox: &Mailbox) -> vec3<f32> {
        let circle_pos = vec2(self.config.earth_radius, 0.0).rotate(mailbox.latitude);
        vec3(mailbox.x, circle_pos.x, -circle_pos.y)
    }
    pub fn hovered_mailbox(&self) -> Option<usize> {
        // self.hovered_mailbox();
        let ray = self.camera.pixel_ray(self.framebuffer_size, self.cursor);
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
            cursor: vec2::ZERO,
        }
    }

    fn restart(&mut self) {
        self.transition = Some(geng::state::Transition::Switch(Box::new(Game::new(
            &self.geng,
            &self.assets,
            &self.config,
        ))));
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, .. } => {
                self.cursor = position.map(|x| x as f32);
                self.start_drag();
            }
            geng::Event::MouseMove { position, .. } => {
                self.cursor = position.map(|x| x as f32);
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                self.cursor = position.map(|x| x as f32);
                self.end_drag();
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.restart();
            }
            geng::Event::KeyDown { .. } => {
                self.start_drag();
            }
            geng::Event::KeyUp { .. } => {
                self.end_drag();
            }
            geng::Event::TouchStart { touches } => {
                self.end_drag();
                if let Some(touch) = touches.last() {
                    self.cursor = touch.position.map(|x| x as f32);
                    self.start_drag();
                }
            }
            geng::Event::TouchMove { touches } => {
                if let Some(touch) = touches.last() {
                    self.cursor = touch.position.map(|x| x as f32);
                }
            }
            geng::Event::TouchEnd { .. } => {
                self.end_drag();
            }
            _ => {}
        }
    }
    fn update(&mut self, delta_time: f64) {
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

fn main() {
    let geng = Geng::new("Juggle Mail - by kuviman for LD53");
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
