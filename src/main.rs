use geng::prelude::*;

#[derive(Deserialize)]
pub struct Config {
    pub gravity: f32,
    pub throw_speed: f32,
    pub throw_angle: f32,
    pub item_scale: f32,
    pub item_hold_scale: f32,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    envelope: Rc<ugli::Texture>,
    bag: ugli::Texture,
}

struct Item {
    texture: Rc<ugli::Texture>,
    pos: vec2<f32>,
    vel: vec2<f32>,
    rot: f32,
    w: f32,
    half_size: vec2<f32>,
}

impl Item {
    pub fn new(texture: &Rc<ugli::Texture>, scale: f32) -> Self {
        Self {
            texture: texture.clone(),
            pos: vec2::ZERO,
            vel: vec2::ZERO,
            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
            w: thread_rng().gen_range(-1.0..1.0),
            half_size: vec2(texture.size().map(|x| x as f32).aspect(), 1.0) * scale,
        }
    }
}

struct Game {
    framebuffer_size: vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    config: Rc<Config>,
    camera: geng::Camera2d,
    items: Vec<Item>,
    bag_position: Aabb2<f32>,
    holding: Option<Item>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Rc<Config>) -> Self {
        Self {
            framebuffer_size: vec2::splat(1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            camera: geng::Camera2d {
                center: vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            },
            items: vec![],
            bag_position: Aabb2::point(vec2(-1.0, -1.0)).extend_uniform(1.0),
            holding: None,
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
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if self.bag_position.contains(pos) {
                    self.holding = Some(Item::new(&self.assets.envelope, self.config.item_scale));
                } else if let Some(index) = self.items.iter().rposition(|item| {
                    (Quad::unit()
                        .scale(item.half_size)
                        .rotate(item.rot)
                        .translate(item.pos)
                        .transform
                        .inverse()
                        * pos.extend(1.0))
                    .into_2d()
                }) {
                }
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if let Some(mut item) = self.holding.take() {
                    item.pos = pos;
                    item.vel = vec2(0.0, self.config.throw_speed).rotate(thread_rng().gen_range(
                        -self.config.throw_angle.to_radians()..self.config.throw_angle.to_radians(),
                    ));
                    self.items.push(item);
                }
            }
            _ => {}
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        for item in &mut self.items {
            item.vel.y -= self.config.gravity * delta_time;
            item.pos += item.vel * delta_time;
            item.rot += item.w * delta_time;
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        let mouse_pos = self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().cursor_position().map(|x| x as f32),
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::TexturedQuad::new(self.bag_position, &self.assets.bag),
        );
        if let Some(item) = &self.holding {
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::unit(&*item.texture)
                    .scale(item.half_size * self.config.item_hold_scale)
                    .rotate(item.rot)
                    .translate(mouse_pos),
            );
        }
        for item in &self.items {
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::unit(&*item.texture)
                    .scale(item.half_size)
                    .rotate(item.rot)
                    .translate(item.pos),
            );
        }
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
