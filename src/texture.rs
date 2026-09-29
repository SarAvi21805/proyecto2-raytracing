use crate::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    ParisStone,
    Brick,
    RoofTile,
    Glass,
    Ladybug,
    CatNoir,
    Akuma,
    GoldLight,
    WetStreet,
    Metal,
}

impl TextureKind {
    pub fn sample(self, u: f32, v: f32, point: Vec3) -> Vec3 {
        let u = fract(u);
        let v = fract(v);
        match self {
            Self::ParisStone => {
                let variation = smooth_noise(u * 5.0, v * 5.0);
                let seams = mortar_mask(u * 3.0, v * 3.0, 0.055);
                Vec3::new(0.82, 0.77, 0.68) * (0.92 + variation * 0.10)
                    + Vec3::new(0.12, 0.11, 0.10) * seams
            }
            Self::Brick => {
                let row = (v * 5.0).floor() as i32;
                let shifted_u = u * 4.0 + if row % 2 == 0 { 0.0 } else { 0.5 };
                let mortar = mortar_mask(shifted_u, v * 5.0, 0.075);
                let variation = smooth_noise(shifted_u * 1.4, v * 6.0);
                let brick = Vec3::new(0.58, 0.22, 0.15) * (0.88 + 0.16 * variation);
                brick.lerp(Vec3::new(0.60, 0.55, 0.50), mortar)
            }
            Self::RoofTile => {
                let rows = v * 5.0;
                let columns = u * 6.0 + if rows.floor() as i32 % 2 == 0 { 0.0 } else { 0.5 };
                let seam = mortar_mask(columns, rows, 0.065);
                let wave = ((u * std::f32::consts::TAU * 3.0).sin() * 0.5 + 0.5) * 0.06;
                (Vec3::new(0.20, 0.25, 0.34) + Vec3::ONE * wave)
                    .lerp(Vec3::new(0.06, 0.08, 0.12), seam)
            }
            Self::Glass => {
                let frame = edge_mask(u, v, 0.065);
                Vec3::new(0.66, 0.86, 0.96).lerp(Vec3::new(0.95, 1.0, 1.0), frame)
            }
            Self::Ladybug => {
                let cell_u = fract(u * 2.0);
                let cell_v = fract(v * 2.0);
                let dx = cell_u - 0.5;
                let dy = cell_v - 0.5;
                let spot = smoothstep(0.16, 0.12, (dx * dx + dy * dy).sqrt());
                let seam = smoothstep(0.025, 0.055, (u - 0.5).abs());
                let red = Vec3::new(0.88, 0.025, 0.045) * (0.94 + seam * 0.06);
                red.lerp(Vec3::new(0.012, 0.014, 0.020), spot)
            }
            Self::CatNoir => {
                let satin = ((u * 7.0 + v * 2.0).sin() * 0.5 + 0.5) * 0.045;
                let green_line = smoothstep(0.035, 0.0, (v - 0.50).abs());
                (Vec3::new(0.018, 0.022, 0.028) + Vec3::ONE * satin)
                    .lerp(Vec3::new(0.18, 0.95, 0.28), green_line * 0.42)
            }
            Self::Akuma => {
                let swirl = ((u * 12.0 + point.y * 0.55).sin()
                    * (v * 10.0 - point.x * 0.35).cos())
                    * 0.5
                    + 0.5;
                Vec3::new(0.28, 0.025, 0.50)
                    .lerp(Vec3::new(0.92, 0.34, 1.0), swirl)
            }
            Self::GoldLight => {
                let glow = ((u * std::f32::consts::TAU).sin().abs() * 0.18) + 0.82;
                Vec3::new(1.0, 0.54, 0.08) * glow
            }
            Self::WetStreet => {
                let lane = smoothstep(0.025, 0.0, (fract(u * 2.0) - 0.5).abs());
                let variation = smooth_noise(u * 4.0 + point.x * 0.08, v * 4.0 + point.z * 0.08);
                (Vec3::new(0.075, 0.09, 0.13) * (0.94 + variation * 0.10))
                    .lerp(Vec3::new(0.58, 0.44, 0.18), lane * 0.38)
            }
            Self::Metal => {
                let brushed = ((v * 22.0 + point.y * 0.35).sin() * 0.5 + 0.5) * 0.08;
                Vec3::new(0.31, 0.25, 0.21) + Vec3::ONE * brushed
            }
        }
    }

    pub fn tangent_normal(self, u: f32, v: f32, strength: f32) -> Vec3 {
        if strength <= 0.0 {
            return Vec3::new(0.0, 0.0, 1.0);
        }

        let e = 0.0125;
        let h = self.height(u, v);
        let hx = self.height(u + e, v);
        let hy = self.height(u, v + e);
        Vec3::new((h - hx) * strength / e, (h - hy) * strength / e, 1.0).normalized()
    }

    fn height(self, u: f32, v: f32) -> f32 {
        match self {
            Self::ParisStone => {
                ((u * std::f32::consts::TAU * 3.0).sin()
                    * (v * std::f32::consts::TAU * 3.0).sin())
                    * 0.18
            }
            Self::Brick => {
                ((u * std::f32::consts::TAU * 4.0).sin().abs()
                    * (v * std::f32::consts::TAU * 5.0).sin().abs())
                    * 0.25
            }
            Self::RoofTile => {
                (u * std::f32::consts::TAU * 6.0).sin() * 0.22
                    + (v * std::f32::consts::TAU * 5.0).sin() * 0.08
            }
            Self::WetStreet => {
                (u * std::f32::consts::TAU * 3.0).sin()
                    * (v * std::f32::consts::TAU * 2.0).cos()
                    * 0.08
            }
            Self::Ladybug => {
                (u * std::f32::consts::TAU * 2.0).sin()
                    * (v * std::f32::consts::TAU * 2.0).sin()
                    * 0.04
            }
            Self::CatNoir | Self::Akuma | Self::Metal => {
                (u * std::f32::consts::TAU * 4.0).sin() * 0.03
            }
            _ => 0.0,
        }
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

fn edge_mask(u: f32, v: f32, width: f32) -> f32 {
    let distance = u.min(v).min(1.0 - u).min(1.0 - v);
    smoothstep(width, width * 0.45, distance)
}

fn mortar_mask(u: f32, v: f32, width: f32) -> f32 {
    let du = fract(u).min(1.0 - fract(u));
    let dv = fract(v).min(1.0 - fract(v));
    smoothstep(width, width * 0.55, du.min(dv))
}

fn smooth_noise(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = smooth_curve(fract(x));
    let ty = smooth_curve(fract(y));
    let a = hash2(x0, y0);
    let b = hash2(x0 + 1, y0);
    let c = hash2(x0, y0 + 1);
    let d = hash2(x0 + 1, y0 + 1);
    lerp(lerp(a, b, tx), lerp(c, d, tx), ty)
}

fn hash2(x: i32, y: i32) -> f32 {
    let mut value = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    (value & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

fn smooth_curve(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let denominator = edge1 - edge0;
    if denominator.abs() <= f32::EPSILON {
        return if value < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((value - edge0) / denominator).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_procedural_textures_return_finite_colors() {
        let textures = [
            TextureKind::ParisStone,
            TextureKind::Brick,
            TextureKind::RoofTile,
            TextureKind::Glass,
            TextureKind::Ladybug,
            TextureKind::CatNoir,
            TextureKind::Akuma,
            TextureKind::GoldLight,
            TextureKind::WetStreet,
            TextureKind::Metal,
        ];
        for texture in textures {
            let color = texture.sample(0.37, 0.61, Vec3::new(3.0, 8.0, 5.0));
            assert!(color.x.is_finite() && color.y.is_finite() && color.z.is_finite());
        }
    }
}