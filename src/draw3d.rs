use super::*;

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_uv: vec2<f32>,
}

#[derive(ugli::Vertex)]
struct SpriteVertex {
    a_pos: vec2<f32>,
}

pub struct Draw3d {
    assets: Rc<Assets>,
    quad: ugli::VertexBuffer<SpriteVertex>,
}

impl Draw3d {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            assets: assets.clone(),
            quad: ugli::VertexBuffer::new_static(
                geng.ugli(),
                vec![
                    SpriteVertex {
                        a_pos: vec2(0.0, 0.0),
                    },
                    SpriteVertex {
                        a_pos: vec2(1.0, 0.0),
                    },
                    SpriteVertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    SpriteVertex {
                        a_pos: vec2(0.0, 1.0),
                    },
                ],
            ),
        }
    }
    pub fn draw_sprite_with_transform(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        texture: &ugli::Texture,
        transform: mat4<f32>,
        color: Rgba<f32>,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.sprite,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_model_matrix: transform,
                    u_color: color,
                },
                camera.uniforms(framebuffer.size().map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }
    pub fn draw_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        texture: &ugli::Texture,
        pos: vec3<f32>,
        size: vec2<f32>,
        color: Rgba<f32>,
    ) {
        self.draw_sprite_with_transform(
            framebuffer,
            camera,
            texture,
            mat4::translate(pos)
                * mat4::rotate_x(-camera.latitude - camera.rot)
                * mat4::scale(size.extend(1.0))
                * mat4::translate(vec3(-0.5, 0.0, 0.0)),
            color,
        );
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        data: &ugli::VertexBuffer<Vertex>,
        mode: ugli::DrawMode,
        texture: &ugli::Texture,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.mesh3d,
            mode,
            data,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_model_matrix: mat4::identity(),
                },
                camera.uniforms(framebuffer.size().map(|x| x as f32)),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }
}
