use super::*;

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
    fn load(manager: &geng::Manager, path: &std::path::Path) -> geng::asset::Future<Self> {
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
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub shaders: Shaders,
    pub sfx: Sfx,
    pub envelope: Rc<Texture>,
    pub bag: Texture,
    pub hand: Texture,
    pub holding_hand: Texture,
    pub mailbox: Texture,
    #[load(postprocess = "make_repeated")]
    pub road: Texture,
}

fn make_repeated(texture: &mut Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
}
