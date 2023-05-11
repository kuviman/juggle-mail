#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use geng::prelude::*;

mod assets;
mod camera;
mod config;
mod draw3d;
mod final_screen;
mod font;
mod game;
mod leaderboard;
mod leaderboard_screen;
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

#[derive(clap::Parser)]
struct Args {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();
    let args: Args = cli::parse();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Juggle Mail - by kuviman for LD53".to_owned(),
        target_ui_resolution: Some(vec2(800.0, 600.0)),
        ..geng::ContextOptions::from_args(&args.geng)
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
