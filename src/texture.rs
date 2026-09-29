use crate::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    Grass,
    Dirt,
    Stone,
    Sand,
    Glass,
    Lava,
    Water,
    Wood,
}

impl TextureKind {
    pub fn sample(self, u: f32, v: f32, point: Vec3) -> Vec3 {
        let u = fract(u);
        let v = fract(v);
        match self {
            Self::Grass => {
                let blades = hash2((u * 12.0) as i32, (v * 12.0) as i32);
                Vec3::new(0.24, 0.68, 0.18) * (0.88 + 0.15 * blades)
            }
            Self::Dirt => {
                let grains = hash2((u * 32.0) as i32, (v * 32.0) as i32);
                Vec3::new(0.50, 0.29, 0.13) * (0.76 + 0.32 * grains)
            }
            Self::Stone => {
                let veins = ((u * 19.0 + (v * 13.0).sin() * 2.0).sin() * 0.5 + 0.5)
                    .powf(5.0);
                let grain = hash2((u * 40.0) as i32, (v * 40.0) as i32);
                Vec3::new(0.47, 0.50, 0.54) * (0.70 + grain * 0.20)
                    + Vec3::new(0.19, 0.22, 0.25) * veins
            }
            Self::Sand => {
                let ripples = ((u * 42.0 + (v * 9.0).sin()).sin() * 0.5 + 0.5) * 0.16;
                Vec3::new(0.88, 0.75, 0.43) * (0.88 + ripples)
            }
            Self::Glass => {
                let border = edge_mask(u, v, 0.075);
                Vec3::new(0.55, 0.88, 0.94).lerp(Vec3::new(0.90, 1.0, 1.0), border)
            }
            Self::Lava => {
                let flow = ((u * 16.0 + (point.z * 0.8).sin()).sin()
                    * (v * 18.0 + (point.x * 0.7).cos()).sin())
                    .abs();
                Vec3::new(1.0, 0.10, 0.01).lerp(Vec3::new(1.0, 0.82, 0.08), flow)
            }
            Self::Water => {
                let wave = ((u * 24.0 + point.z).sin() + (v * 21.0 + point.x).cos()) * 0.05;
                Vec3::new(0.08, 0.38, 0.65) + Vec3::new(wave, wave, wave * 1.5)
            }
            Self::Wood => {
                let rings = ((u * 13.0 + (v * 4.0).sin()).sin() * 0.5 + 0.5) * 0.28;
                Vec3::new(0.42, 0.20, 0.07) + Vec3::new(0.25, 0.12, 0.03) * rings
            }
        }
    }

    pub fn tangent_normal(self, u: f32, v: f32, strength: f32) -> Vec3 {
        if strength <= 0.0 {
            return Vec3::new(0.0, 0.0, 1.0);
        }

        let e = 0.01;
        let h = self.height(u, v);
        let hx = self.height(u + e, v);
        let hy = self.height(u, v + e);
        Vec3::new((h - hx) * strength / e, (h - hy) * strength / e, 1.0).normalized()
    }

    fn height(self, u: f32, v: f32) -> f32 {
        match self {
            Self::Stone => {
                let cells = hash2((fract(u) * 28.0) as i32, (fract(v) * 28.0) as i32);
                cells * 0.7 + ((u * 18.0).sin() * (v * 17.0).cos()).abs() * 0.3
            }
            Self::Water => ((u * 21.0).sin() + (v * 23.0).cos()) * 0.5,
            Self::Grass => hash2((fract(u) * 40.0) as i32, (fract(v) * 40.0) as i32),
            _ => 0.0,
        }
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

fn edge_mask(u: f32, v: f32, width: f32) -> f32 {
    if u < width || v < width || u > 1.0 - width || v > 1.0 - width {
        1.0
    } else {
        0.0
    }
}

fn hash2(x: i32, y: i32) -> f32 {
    let mut value = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    (value & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

