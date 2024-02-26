use super::*;

pub struct FinalScreen {
    geng: Geng,
    assets: Rc<Assets>,
    config: Rc<Config>,

    time_scale: usize,
    game_time: usize,
    lives: usize,
    name: String,
    global_place: Option<usize>,

    transition: Option<geng::state::Transition>,

    score: f32,
    top10: Option<Vec<jornet::Score>>,
}

impl FinalScreen {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        config: &Rc<Config>,
        diff: Difficulty,
        score: f32,
        name: String,
        pos_and_scores: Option<(usize, Vec<jornet::Score>)>,
    ) -> Self {
        let (global_place, top10) = match pos_and_scores {
            Some((pos, scores)) => (Some(pos), Some(scores)),
            None => (None, None),
        };
        Self {
            top10,
            global_place,
            name,
            score,
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            time_scale: config
                .time_scale
                .iter()
                .position(|x| *x == diff.time_scale)
                .unwrap(),
            game_time: config
                .game_time
                .iter()
                .position(|x| *x == diff.game_time)
                .unwrap(),
            lives: config.lives.iter().position(|x| *x == diff.lives).unwrap(),
            transition: None,
        }
    }
}

impl geng::State for FinalScreen {
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
        let game_time = ui::TextureWidget::new(&self.assets.difficulty.game_time[self.game_time]);
        let time_scale =
            ui::TextureWidget::new(&self.assets.difficulty.time_scale[self.time_scale]);
        let lives = ui::TextureWidget::new(&self.assets.difficulty.lives[self.lives]);
        let play = ui::TextureButton::new(cx, &self.assets.play_again, &self.assets.ui_sfx);
        if play.was_clicked() {
            self.transition = Some(geng::state::Transition::Switch(Box::new(Game::new(
                &self.geng,
                &self.assets,
                &self.config,
                Difficulty {
                    time_scale: self.config.time_scale[self.time_scale],
                    game_time: self.config.game_time[self.game_time],
                    lives: self.config.lives[self.lives],
                },
                self.name.clone(),
            ))));
        }
        let score = ui::Text::new(&self.assets.font, (self.score.floor() as i32).to_string());
        let menu = ui::TextureButton::new(cx, &self.assets.menu, &self.assets.ui_sfx);
        if menu.was_clicked() {
            self.transition = Some(geng::state::Transition::Pop);
        }
        let mut stack = stack![
            ui::TextureWidget::new(&self.assets.final_screen),
            game_time.place(300, 95),
            time_scale.place(300, 133),
            lives.place(300, 170),
            menu.place(25, 235),
            play.place(180, 220),
            score.fixed_size(vec2(0.0, 16.0)).place(90, 163),
        ];

        #[cfg(feature = "leaderboard")]
        {
            if let Some(global_place) = self.global_place {
                let global_place = ui::Text::new(&self.assets.font, (global_place + 1).to_string());
                stack.push(Box::new(
                    global_place.fixed_size(vec2(0.0, 16.0)).place(53, 208),
                ));
            }
            if let Some(top10) = &self.top10 {
                let leaderboard_button = ui::TextureButton::new(
                    cx,
                    &self.assets.leaderboard_button,
                    &self.assets.ui_sfx,
                );
                if leaderboard_button.was_clicked() {
                    self.transition = Some(geng::state::Transition::Push(Box::new(
                        crate::leaderboard_screen::LeaderboardScreen::new(
                            &self.geng,
                            &self.assets,
                            top10.clone(),
                        ),
                    )));
                }
                stack.push(Box::new(leaderboard_button.place(70, 190)));
            }
        }

        stack.center().boxed()
    }
}
