use super::*;

pub struct Camera {
    near: f32,
    fov: f32,
    cam2d: geng::Camera2d,
    pub latitude: f32,
    pub rot: f32,
    height: f32,
}

impl Camera {
    pub fn new(fov: f32, ui_fov: f32, rot: f32, height: f32) -> Self {
        Self {
            near: 0.1,
            fov,
            cam2d: geng::Camera2d {
                fov: ui_fov,
                center: vec2::ZERO,
                rotation: 0.0,
            },
            latitude: 0.0,
            rot,
            height,
        }
    }
    pub fn as_2d(&self) -> &geng::Camera2d {
        &self.cam2d
    }
    pub fn fov(&self) -> f32 {
        self.cam2d.fov
    }
}

impl geng::AbstractCamera3d for Camera {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::rotate_x(self.rot)
            * mat4::translate(vec3(0.0, -self.height, 0.0))
            * mat4::rotate_x(self.latitude)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        mat4::perspective(self.fov, framebuffer_size.aspect(), self.near, 100.0)
    }
}
