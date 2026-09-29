use crate::{
    material::{default_materials, Material},
    math::Vec3,
    world::{generate_diorama, VoxelWorld},
};

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

pub struct Scene {
    pub world: VoxelWorld,
    pub materials: Vec<Material>,
    pub lights: Vec<PointLight>,
}

impl Scene {
    pub fn diorama(seed: u32) -> Self {
        Self {
            world: generate_diorama(seed),
            materials: default_materials(),
            lights: vec![
                PointLight {
                    position: Vec3::new(10.0, 22.0, 6.0),
                    color: Vec3::new(0.58, 0.68, 1.0),
                    intensity: 58.0,
                },
                PointLight {
                    position: Vec3::new(16.0, 9.5, 14.0),
                    color: Vec3::new(0.72, 0.18, 1.0),
                    intensity: 25.0,
                },
                PointLight {
                    position: Vec3::new(5.0, 16.0, 26.0),
                    color: Vec3::new(1.0, 0.63, 0.20),
                    intensity: 32.0,
                },
            ],
        }
    }
}