use crate::{math::Vec3, texture::TextureKind};

pub const GRASS: usize = 0;
pub const DIRT: usize = 1;
pub const STONE: usize = 2;
pub const SAND: usize = 3;
pub const GLASS: usize = 4;
pub const LAVA: usize = 5;
pub const WATER: usize = 6;
pub const WOOD: usize = 7;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub name: &'static str,
    pub albedo: Vec3,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub emission: Vec3,
    pub normal_strength: f32,
    pub texture: TextureKind,
}

pub fn default_materials() -> Vec<Material> {
    vec![
        Material {
            name: "Pasto",
            albedo: Vec3::new(0.70, 1.00, 0.65),
            specular: 0.08,
            shininess: 12.0,
            transparency: 0.0,
            reflectivity: 0.02,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.22,
            texture: TextureKind::Grass,
        },
        Material {
            name: "Tierra",
            albedo: Vec3::new(0.85, 0.72, 0.58),
            specular: 0.03,
            shininess: 5.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.12,
            texture: TextureKind::Dirt,
        },
        Material {
            name: "Piedra con mapa normal",
            albedo: Vec3::new(0.88, 0.90, 0.95),
            specular: 0.30,
            shininess: 48.0,
            transparency: 0.0,
            reflectivity: 0.10,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.55,
            texture: TextureKind::Stone,
        },
        Material {
            name: "Arena",
            albedo: Vec3::new(1.0, 0.95, 0.72),
            specular: 0.06,
            shininess: 8.0,
            transparency: 0.0,
            reflectivity: 0.02,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.08,
            texture: TextureKind::Sand,
        },
        Material {
            name: "Vidrio refractivo",
            albedo: Vec3::new(0.75, 0.95, 1.0),
            specular: 0.95,
            shininess: 160.0,
            transparency: 0.82,
            reflectivity: 0.08,
            refractive_index: 1.50,
            emission: Vec3::ZERO,
            normal_strength: 0.0,
            texture: TextureKind::Glass,
        },
        Material {
            name: "Lava emisiva",
            albedo: Vec3::new(1.0, 0.30, 0.05),
            specular: 0.15,
            shininess: 18.0,
            transparency: 0.0,
            reflectivity: 0.03,
            refractive_index: 1.0,
            emission: Vec3::new(4.2, 1.1, 0.08),
            normal_strength: 0.10,
            texture: TextureKind::Lava,
        },
        Material {
            name: "Agua",
            albedo: Vec3::new(0.32, 0.68, 0.95),
            specular: 0.90,
            shininess: 110.0,
            transparency: 0.68,
            reflectivity: 0.16,
            refractive_index: 1.333,
            emission: Vec3::ZERO,
            normal_strength: 0.18,
            texture: TextureKind::Water,
        },
        Material {
            name: "Madera",
            albedo: Vec3::new(0.92, 0.74, 0.50),
            specular: 0.10,
            shininess: 16.0,
            transparency: 0.0,
            reflectivity: 0.02,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.12,
            texture: TextureKind::Wood,
        },
    ]
}

