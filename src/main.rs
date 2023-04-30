use geng::prelude::*;

mod assets;
mod camera;
mod config;
mod draw3d;
mod font;
mod util;
mod game;

use game::Game;
use assets::*;
use camera::*;
use config::*;
use draw3d::Draw3d;
use font::*;
use util::*;

fn main() {
    let geng = Geng::new("Juggle Mail - by kuviman for LD53");
    geng.window().set_cursor_type(geng::CursorType::None);
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
