use crate::{math::Vec3, ray::Ray};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub vertical_fov_degrees: f32,
}

impl Camera {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            yaw: 0.75,
            pitch: 0.42,
            distance: 32.0,
            vertical_fov_degrees: 50.0,
        }
    }

    pub fn position(self) -> Vec3 {
        let horizontal = self.distance * self.pitch.cos();
        self.target
            + Vec3::new(
                horizontal * self.yaw.sin(),
                self.distance * self.pitch.sin(),
                horizontal * self.yaw.cos(),
            )
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-0.15, 1.25);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(10.0, 55.0);
    }

    pub fn ray_for_pixel(self, x: usize, y: usize, width: usize, height: usize) -> Ray {
        let position = self.position();
        let forward = (self.target - position).normalized();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right = forward.cross(world_up).normalized();
        let up = right.cross(forward).normalized();

        let aspect = width as f32 / height as f32;
        let half_height = (self.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = aspect * half_height;
        let screen_x = (2.0 * (x as f32 + 0.5) / width as f32 - 1.0) * half_width;
        let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / height as f32) * half_height;

        Ray::new(position, forward + right * screen_x + up * screen_y)
    }
}

