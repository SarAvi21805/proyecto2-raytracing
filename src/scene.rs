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
                    position: Vec3::new(8.0, 18.0, 7.0),
                    color: Vec3::new(1.0, 0.88, 0.68),
                    intensity: 115.0,
                },
                PointLight {
                    position: Vec3::new(17.5, 10.0, 16.5),
                    color: Vec3::new(1.0, 0.20, 0.04),
                    intensity: 42.0,
                },
            ],
        }
    }
}

