use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    envelope: Rc<ugli::Texture>,
}

struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    camera: geng::Camera2d,
    items: Vec<Item>,
}

struct Item {
    texture: Rc<ugli::Texture>,
    pos: vec2<f32>,
    rot: f32,
    half_size: vec2<f32>,
}

impl Item {
    pub fn new(texture: &Rc<ugli::Texture>, pos: vec2<f32>) -> Self {
        Self {
            texture: texture.clone(),
            pos,
            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
            half_size: vec2(texture.size().map(|x| x as f32).aspect(), 1.0),
        }
    }
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            },
            items: vec![Item::new(&assets.envelope, vec2::ZERO)],
        }
    }
}

impl geng::State for Game {
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
        Game::new(&geng, &assets)
    })
}
