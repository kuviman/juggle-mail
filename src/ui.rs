use super::*;

pub struct TextureButton<'a> {
    sense: &'a mut geng::ui::Sense,
    clicked: bool,
    texture: &'a ugli::Texture,
    hover_texture: &'a ugli::Texture,
    size: vec2<f64>,
}

impl<'a> TextureButton<'a> {
    pub fn new(cx: &'a geng::ui::Controller, texture: &'a ugli::Texture, sfx: &'a UiSfx) -> Self {
        Self::new2(cx, texture, texture, sfx, texture.size().map(|x| x as f64))
    }
    pub fn new2(
        cx: &'a geng::ui::Controller,
        texture: &'a ugli::Texture,
        hover_texture: &'a ugli::Texture,
        sfx: &'a UiSfx,
        size: vec2<f64>,
    ) -> Self {
        let sense: &'a mut geng::ui::Sense = cx.get_state();
        let clicked = sense.take_clicked();
        if clicked {
            sfx.click.play();
        }
        let last_hover: &'a mut bool = cx.get_state();
        if *last_hover != sense.is_hovered() {
            *last_hover = sense.is_hovered();
            sfx.hover.play();
        }
        Self {
            clicked,
            sense,
            texture,
            hover_texture,
            size,
        }
    }
    pub fn was_clicked(&self) -> bool {
        self.clicked
    }
}

impl geng::ui::Widget for TextureButton<'_> {
    fn sense(&mut self) -> Option<&mut geng::ui::Sense> {
        Some(self.sense)
    }
    fn calc_constraints(&mut self, _cx: &geng::ui::ConstraintsContext) -> geng::ui::Constraints {
        geng::ui::Constraints {
            min_size: self.size,
            flex: vec2::ZERO,
        }
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        let extra = 0.1;
        let size = if self.sense.is_captured() {
            1.0 - extra
        } else if self.sense.is_hovered() {
            1.0 + extra
        } else {
            1.0
        };
        cx.draw2d.draw2d(
            cx.framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::TexturedQuad::unit(if self.sense.is_hovered() {
                self.hover_texture
            } else {
                self.texture
            })
            .scale_uniform(size)
            .scale(cx.position.size().map(|x| x as f32 / 2.0))
            .translate(cx.position.center().map(|x| x as f32)),
        );
    }
}

pub struct TextureWidget<'a> {
    texture: &'a ugli::Texture,
    size: vec2<f64>,
}

impl<'a> TextureWidget<'a> {
    pub fn new(texture: &'a ugli::Texture) -> Self {
        Self {
            texture,
            size: texture.size().map(|x| x as f64),
        }
    }
}

impl geng::ui::Widget for TextureWidget<'_> {
    fn calc_constraints(&mut self, _cx: &geng::ui::ConstraintsContext) -> geng::ui::Constraints {
        geng::ui::Constraints {
            min_size: self.size,
            flex: vec2::ZERO,
        }
    }
    fn draw(&mut self, cx: &mut geng::ui::DrawContext) {
        cx.draw2d.draw2d(
            cx.framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::TexturedQuad::new(cx.position.map(|x| x as f32), self.texture),
        );
    }
}

pub struct Place<T> {
    inner: T,
    pos: vec2<f64>,
}

impl<T: geng::ui::Widget> geng::ui::Widget for Place<T> {
    fn calc_constraints(
        &mut self,
        children: &geng::ui::ConstraintsContext,
    ) -> geng::ui::Constraints {
        children.get_constraints(&self.inner)
    }

    fn walk_children_mut(&mut self, f: &mut dyn FnMut(&mut dyn geng::ui::Widget)) {
        f(&mut self.inner);
    }

    fn layout_children(&mut self, cx: &mut geng::ui::LayoutContext) {
        let child_c = cx.get_constraints(&self.inner);
        cx.set_position(
            &self.inner,
            Aabb2::point(cx.position.top_left() + self.pos + vec2(0.0, -child_c.min_size.y))
                .extend_positive(child_c.min_size),
        );
    }
}

pub trait WidgetExt: geng::ui::Widget + Sized {
    fn place(self, x: i32, y: i32) -> Place<Self> {
        Place {
            inner: self,
            pos: vec2(x, -y).map(|x| x as f64),
        }
    }
}

impl<T: geng::ui::Widget> WidgetExt for T {}
