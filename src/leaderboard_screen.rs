use super::*;

pub struct LeaderboardScreen {
    geng: Geng,
    assets: Rc<Assets>,
    top10: Vec<jornet::Score>,
    transition: Option<geng::state::Transition>,
}

impl LeaderboardScreen {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, top10: Vec<jornet::Score>) -> Self {
        Self {
            top10,
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
        }
    }
}

impl geng::State for LeaderboardScreen {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.geng
            .window()
            .set_cursor_type(geng::CursorType::Default);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let back = ui::TextureButton::new(cx, &self.assets.back, &self.assets.ui_sfx);
        if back.was_clicked() {
            self.transition = Some(geng::state::Transition::Pop);
        }
        let mut stack = stack![
            ui::TextureWidget::new(&self.assets.leaderboard_background),
            back.place(25, 235),
        ];
        let mut y = 140;
        for (rank, score) in self.top10.iter().enumerate() {
            let rank = rank + 1;
            let name = &score.player;
            let score = (score.score.floor() as i32).to_string();
            let name = ui::Text::left_align(&self.assets.font, format!("#{rank}: {name}"));
            stack.push(Box::new(name.fixed_size(vec2(0.0, 16.0)).place(25, y)));
            let score = ui::Text::right_align(&self.assets.font, score);
            stack.push(Box::new(score.fixed_size(vec2(0.0, 16.0)).place(370, y)));
            y += 17;
        }
        stack.center().boxed()
    }
}
