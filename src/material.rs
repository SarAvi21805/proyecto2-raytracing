use crate::{math::Vec3, texture::TextureKind};

pub const PARIS_STONE: usize = 0;
pub const BRICK: usize = 1;
pub const ROOFTOP: usize = 2;
pub const GLASS: usize = 3;
pub const LADYBUG: usize = 4;
pub const CAT_NOIR: usize = 5;
pub const AKUMA: usize = 6;
pub const GOLD_LIGHT: usize = 7;
pub const WET_STREET: usize = 8;
pub const METAL: usize = 9;

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
            name: "Piedra parisina",
            albedo: Vec3::new(0.90, 0.86, 0.78),
            specular: 0.16,
            shininess: 30.0,
            transparency: 0.0,
            reflectivity: 0.05,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.22,
            texture: TextureKind::ParisStone,
        },
        Material {
            name: "Ladrillo con mapa normal",
            albedo: Vec3::new(0.96, 0.78, 0.68),
            specular: 0.10,
            shininess: 22.0,
            transparency: 0.0,
            reflectivity: 0.03,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.18,
            texture: TextureKind::Brick,
        },
        Material {
            name: "Taja parisina mojada",
            albedo: Vec3::new(0.72, 0.76, 0.86),
            specular: 0.72,
            shininess: 96.0,
            transparency: 0.0,
            reflectivity: 0.30,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.16,
            texture: TextureKind::RoofTile,
        },
        Material {
            name: "Claraboya de vidrio refractivo",
            albedo: Vec3::new(0.72, 0.88, 1.0),
            specular: 0.92,
            shininess: 170.0,
            transparency: 0.82,
            reflectivity: 0.10,
            refractive_index: 1.5,
            emission: Vec3::ZERO,
            normal_strength: 0.0,
            texture: TextureKind::Glass,
        },
        Material {
            name: "Ladybug rojo con lunares",
            albedo: Vec3::ONE,
            specular: 0.58,
            shininess: 88.0,
            transparency: 0.0,
            reflectivity: 0.17,
            refractive_index: 1.00,
            emission: Vec3::ZERO,
            normal_strength: 0.04,
            texture: TextureKind::Ladybug,
        },
        Material {
            name: "Cat Noir negro satinado",
            albedo: Vec3::ONE,
            specular: 0.86,
            shininess: 132.0,
            transparency: 0.0,
            reflectivity: 0.34,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.03,
            texture: TextureKind::CatNoir,
        },
        Material {
            name: "Akuma magico emisivo",
            albedo: Vec3::new(0.82, 0.62, 1.0),
            specular: 0.52,
            shininess: 72.0,
            transparency: 0.12,
            reflectivity: 0.10,
            refractive_index: 1.18,
            emission: Vec3::new(2.5, 0.35, 3.2),
            normal_strength: 0.03,
            texture: TextureKind::Akuma,
        },
        Material {
            name: "Luz dorada de Paris",
            albedo: Vec3::new(1.0, 0.78, 0.30),
            specular: 0.35,
            shininess: 54.0,
            transparency: 0.0,
            reflectivity: 0.12,
            refractive_index: 1.0,
            emission: Vec3::new(2.1, 1.25, 0.28),
            normal_strength: 0.0,
            texture: TextureKind::GoldLight,
        },
        Material {
            name: "Pavimento mojado reflectante",
            albedo: Vec3::new(0.54, 0.61, 0.74),
            specular: 0.92,
            shininess: 150.0,
            transparency: 0.0,
            reflectivity: 0.46,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.06,
            texture: TextureKind::WetStreet,
        },
        Material {
            name: "Metal de la Torre Eiffel",
            albedo: Vec3::new(0.62, 0.55, 0.48),
            specular: 0.94,
            shininess: 180.0,
            transparency: 0.0,
            reflectivity: 0.62,
            refractive_index: 1.0,
            emission: Vec3::ZERO,
            normal_strength: 0.04,
            texture: TextureKind::Metal,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_exposes_at_least_five_distinct_materials() {
        let materials = default_materials();
        assert!(materials.len() >= 5);
        assert!(materials[GLASS].transparency > 0.0);
        assert!(materials[ROOFTOP].reflectivity > 0.0);
        assert!(materials[BRICK].normal_strength > 0.0);
        assert!(materials[AKUMA].emission.length() > 0.0);
    }
}