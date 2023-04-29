use geng::prelude::*;

#[derive(Deserialize)]
pub struct Config {
    pub gravity: f32,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    envelope: Rc<ugli::Texture>,
}

struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    config: Rc<Config>,
    camera: geng::Camera2d,
    items: Vec<Item>,
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
    pub fn new(texture: &Rc<ugli::Texture>, pos: vec2<f32>, vel: vec2<f32>) -> Self {
        Self {
            texture: texture.clone(),
            pos,
            vel,
            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
            w: thread_rng().gen_range(-1.0..1.0),
            half_size: vec2(texture.size().map(|x| x as f32).aspect(), 1.0),
        }
    }
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Rc<Config>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            camera: geng::Camera2d {
                center: vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            },
            items: vec![Item::new(&assets.envelope, vec2::ZERO, vec2(0.0, 5.0))],
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        for item in &mut self.items {
            item.vel.y -= self.config.gravity * delta_time;
            item.pos += item.vel * delta_time;
            item.rot += item.w * delta_time;
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
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
