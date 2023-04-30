use super::*;

pub struct Particle {
    pos: vec3<f32>,
    pub vel: vec3<f32>,
    t: f32,
    color: Rgba<f32>,
}

impl Game {
    pub fn spawn_particles(&self, pos: vec3<f32>, color: Rgba<f32>) -> Vec<Particle> {
        (0..self.config.particle_count)
            .map(|_| Particle {
                pos,
                vel: vec3(
                    thread_rng().gen_range(-1.0..1.0),
                    thread_rng().gen_range(-1.0..1.0),
                    thread_rng().gen_range(-1.0..1.0),
                ) * self.config.particle_speed,
                t: 0.0,
                color,
            })
            .collect()
    }
    pub fn update_particles(&mut self, delta_time: f32) {
        for p in &mut self.particles_3d {
            p.pos += p.vel * delta_time;
            p.t += delta_time / self.config.particle_lifetime;
        }
        for p in &mut self.particles_ui {
            p.pos += 3.0 * p.vel * delta_time;
            p.t += delta_time / self.config.particle_lifetime;
        }
        self.particles_3d.retain(|p| p.t < 1.0);
        self.particles_ui.retain(|p| p.t < 1.0);
    }
    pub fn draw_particles(&mut self, framebuffer: &mut ugli::Framebuffer) {
        for p in &self.particles_3d {
            self.draw3d.draw_sprite(
                framebuffer,
                &self.camera,
                &self.assets.particle,
                p.pos,
                vec2::splat(self.config.particle_size),
                Rgba {
                    a: 1.0 - p.t,
                    ..p.color
                },
            )
        }
        for p in &self.particles_ui {
            self.geng.draw2d().draw2d(
                framebuffer,
                self.camera.as_2d(),
                &draw2d::TexturedQuad::unit_colored(
                    &self.assets.particle,
                    Rgba {
                        a: 1.0 - p.t,
                        ..p.color
                    },
                )
                .scale_uniform(self.config.particle_size * 2.0)
                .translate(p.pos.xy()),
            )
        }
    }
}
