use super::*;

#[derive(geng::asset::Load)]
pub struct UiSfx {
    #[load(ext = "mp3")]
    pub click: geng::Sound,
    #[load(ext = "mp3")]
    pub hover: geng::Sound,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub sprite: ugli::Program,
    pub mesh3d: ugli::Program,
}

#[derive(Deref, DerefMut)]
pub struct Texture(#[deref] ugli::Texture);

impl std::borrow::Borrow<ugli::Texture> for &Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}

impl geng::asset::Load for Texture {
    type Options = ();
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &(),
    ) -> geng::asset::Future<Self> {
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
pub struct Sfx {
    #[load(ext = "mp3")]
    pub juggle: geng::Sound,
    #[load(ext = "mp3")]
    pub pick: geng::Sound,
    #[load(ext = "mp3")]
    pub throw: geng::Sound,
    #[load(ext = "mp3")]
    pub explosion: geng::Sound,
    #[load(ext = "mp3")]
    pub score: geng::Sound,
    #[load(ext = "mp3")]
    pub lose: geng::Sound,
    #[load(ext = "mp3")]
    pub error: geng::Sound,
    #[load(ext = "mp3")]
    pub timer: geng::Sound,
}

#[derive(geng::asset::Load)]
pub struct DifficultyAssets {
    #[load(listed_in = "_list.ron")]
    pub game_time: Vec<Texture>,
    #[load(listed_in = "_list.ron")]
    pub time_scale: Vec<Texture>,
    #[load(listed_in = "_list.ron")]
    pub lives: Vec<Texture>,
}

#[derive(geng::asset::Load)]
pub struct SkinAssets {
    pub hand: Texture,
    pub holding_hand: Texture,
    pub bike: Texture,
    pub newspaper: Rc<Texture>,
    pub bag: Texture,
}

async fn load_skins(
    manager: &geng::asset::Manager,
    path: impl AsRef<std::path::Path>,
) -> anyhow::Result<HashMap<String, SkinAssets>> {
    let path = path.as_ref();
    let list: Vec<String> = file::load_detect(path.join("_list.ron")).await?;
    Ok(
        future::try_join_all(list.into_iter().map(|name| async move {
            let skin: SkinAssets = manager.load(path.join(&name)).await?;
            Ok::<_, anyhow::Error>((name, skin))
        }))
        .await?
        .into_iter()
        .collect(),
    )
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub difficulty: DifficultyAssets,
    pub ui_sfx: UiSfx,
    pub shaders: Shaders,
    pub sfx: Sfx,
    pub envelope_highlight: Texture,
    #[load(load_with = "load_skins(&manager, base_path.join(\"skins\"))")]
    pub skins: HashMap<String, SkinAssets>,
    pub mailbox: Texture,
    pub aim: Texture,
    #[load(postprocess = "road_postprocess")]
    pub road: Texture,
    #[load(ext = "mp3", options(looped = "true"))]
    pub music: geng::Sound,
    pub particle: Texture,
    pub sun: Texture,
    #[load(listed_in = "_list.ron")]
    pub houses: Vec<Texture>,
    pub heart: Texture,
    pub cross: Texture,
    pub back: Texture,
    pub timer: Texture,
    pub timer_arrow: Texture,
    pub font: Font,
    pub score_background: Texture,
    #[cfg_attr(not(feature = "leaderboard"), load(path = "main_menu_no_lb.png"))]
    pub main_menu: Texture,
    #[cfg_attr(not(feature = "leaderboard"), load(path = "final_screen_no_lb.png"))]
    pub final_screen: Texture,
    pub play_button: Texture,
    pub play_again: Texture,
    pub menu: Texture,
    pub leaderboard_button: Texture,
    pub leaderboard_background: Texture,
}

fn road_postprocess(texture: &mut Texture) {
    texture.set_wrap_mode_separate(ugli::WrapMode::Clamp, ugli::WrapMode::Repeat);
}
