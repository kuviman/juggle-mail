use super::*;

pub trait SoundExt {
    fn play_random_pitch(&self);
}

impl SoundExt for geng::Sound {
    fn play_random_pitch(&self) {
        let mut effect = self.effect();
        const OFFSET: f32 = 0.2;
        effect.set_speed(thread_rng().gen_range(1.0 - OFFSET..1.0 + OFFSET));
        effect.play();
    }
}
