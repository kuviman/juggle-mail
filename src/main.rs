use geng::prelude::*;

mod assets;
mod camera;
mod config;
mod draw3d;
mod font;
mod game;
mod main_menu;
mod ui;
mod util;

use assets::*;
use camera::*;
use config::*;
use draw3d::Draw3d;
use font::*;
use game::Game;
use main_menu::MainMenu;
use ui::WidgetExt;
use util::*;

fn main() {
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Juggle Mail - by kuviman for LD53".to_owned(),
        target_ui_resolution: Some(vec2(800.0, 600.0)),
        ..default()
    });
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
        MainMenu::new(&geng, &assets, &config)
    })
}
