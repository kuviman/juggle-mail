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

pub fn run() {
    logger::init();
    geng::setup_panic_handler();
    let args: Args = cli::parse();
    Geng::run_with(
        &{
            let mut options = geng::ContextOptions {
                window: geng::window::Options::new("Juggle Mail - by kuviman for LD53"),
                target_ui_resolution: Some(vec2(8000.0, 600.0)),
                ..default()
            };
            options.with_cli(&args.geng);
            options
        },
        |geng| async move {
            let assets: Rc<Assets> = geng
                .asset_manager()
                .load(run_dir().join("assets"))
                .await
                .unwrap();
            let config: Config = file::load_detect(run_dir().join("assets").join("config.toml"))
                .await
                .unwrap();
            let config = Rc::new(config);
            geng.run_state(MainMenu::new(&geng, &assets, &config)).await;
        },
    );
}
