use super::*;

pub struct Font {
    textures: HashMap<char, Texture>,
    draw2d: draw2d::Helper,
}

impl Font {
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        text: &str,
        color: Rgba<f32>,
        mut transform: mat3<f32>,
    ) {
        for c in text.chars() {
            self.draw2d.draw2d(
                framebuffer,
                camera,
                &draw2d::TexturedQuad::unit_colored(&self.textures[&c], color).transform(
                    transform * mat3::scale_uniform(0.5) * mat3::translate(vec2::splat(1.0)),
                ),
            );
            transform *= mat3::translate(vec2(1.0, 0.0));
        }
    }
}

impl geng::asset::Load for Font {
    fn load(manager: &geng::Manager, path: &std::path::Path) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let chars: String = file::load_string(path.join("chars.txt")).await?;
            let textures = future::try_join_all(chars.chars().map(|c| {
                let texture = manager.load(path.join(format!("{c}.png")));
                async move { Ok::<_, anyhow::Error>((c, texture.await?)) }
            }))
            .await?
            .into_iter()
            .collect();
            Ok(Self {
                draw2d: draw2d::Helper::new(manager.ugli(), false),
                textures,
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
