use super::*;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    config: Rc<Config>,

    time_scale: usize,
    game_time: usize,
    lives: usize,
    name: String,
    changing_name: bool,

    transition: Option<geng::state::Transition>,
    name_aabb: Aabb2<f64>,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: &Rc<Config>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config: config.clone(),
            time_scale: 0,
            game_time: 0,
            lives: 0,
            transition: None,
            changing_name: false,
            name: preferences::load("name").unwrap_or("you".to_owned()),
            name_aabb: Aabb2::ZERO,
        }
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.geng
            .window()
            .set_cursor_type(geng::CursorType::Default);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::EditText(text) => {
                self.name = text;
                self.name = self.name.to_lowercase();
                self.name.retain(|c| self.assets.font.can_render(c));
                self.name = self.name.chars().take(10).collect();
                self.geng.window().start_text_edit(&self.name);
            }
            geng::Event::TouchStart(geng::Touch { .. }) | geng::Event::MouseDown { .. } => {
                self.geng.window().stop_text_edit();
                self.changing_name = false;
            }
            _ => {}
        }
    }

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let game_time = ui::TextureButton::new(
            cx,
            &self.assets.difficulty.game_time[self.game_time],
            &self.assets.ui_sfx,
        );
        if game_time.was_clicked() {
            self.game_time = (self.game_time + 1) % self.config.game_time.len();
        }
        let time_scale = ui::TextureButton::new(
            cx,
            &self.assets.difficulty.time_scale[self.time_scale],
            &self.assets.ui_sfx,
        );
        if time_scale.was_clicked() {
            self.time_scale = (self.time_scale + 1) % self.config.time_scale.len();
        }
        let lives = ui::TextureButton::new(
            cx,
            &self.assets.difficulty.lives[self.lives],
            &self.assets.ui_sfx,
        );
        if lives.was_clicked() {
            self.lives = (self.lives + 1) % self.config.lives.len();
        }
        let play = ui::TextureButton::new(cx, &self.assets.play_button, &self.assets.ui_sfx);
        if play.was_clicked() {
            self.geng.window().stop_text_edit();
            self.transition = Some(geng::state::Transition::Push(Box::new(Game::new(
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
            preferences::save("name", &self.name);
        }
        let name = ui::TextInput::new(
            cx,
            &mut self.name_aabb,
            self.geng.clone(),
            &self.assets.font,
            self.name.clone(),
            self.changing_name,
            &self.assets.ui_sfx,
        );
        if name.was_clicked() {
            self.geng.window().start_text_edit(&self.name);
            self.changing_name = true;
        }
        stack![
            ui::TextureWidget::new(&self.assets.main_menu),
            game_time.place(300, 95),
            time_scale.place(300, 133),
            lives.place(300, 170),
            play.place(180, 220),
            name.fixed_size(vec2(160.0, 16.0)).place(20, 260),
        ]
        .center()
        .boxed()
    }
}
