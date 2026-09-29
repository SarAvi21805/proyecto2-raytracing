use crate::{camera::Camera, math::Vec3, ray::Ray, scene::Scene, world::Hit};

const EPSILON: f32 = 1.0e-3;

#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub width: usize,
    pub height: usize,
    pub max_bounces: u32,
    pub threads: usize,
}

pub struct Renderer {
    pub settings: RenderSettings,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let threads = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1);
        Self {
            settings: RenderSettings {
                width,
                height,
                max_bounces: 4,
                threads,
            },
        }
    }

    pub fn render(&self, scene: &Scene, camera: Camera) -> Vec<u8> {
        let row_stride = self.settings.width * 4;
        let rows_per_job = self.settings.height.div_ceil(self.settings.threads.max(1));
        let mut pixels = vec![0_u8; row_stride * self.settings.height];

        std::thread::scope(|scope| {
            for (job, chunk) in pixels
                .chunks_mut(row_stride * rows_per_job)
                .enumerate()
            {
                let first_y = job * rows_per_job;
                scope.spawn(move || {
                    for (local_y, row) in chunk.chunks_mut(row_stride).enumerate() {
                        let y = first_y + local_y;
                        for x in 0..self.settings.width {
                            let ray = camera.ray_for_pixel(
                                x,
                                y,
                                self.settings.width,
                                self.settings.height,
                            );
                            let color = self.trace(scene, ray, 0);
                            let rgba = to_rgba8(color);
                            let offset = x * 4;
                            row[offset..offset + 4].copy_from_slice(&rgba);
                        }
                    }
                });
            }
        });

        pixels
    }

    fn trace(&self, scene: &Scene, ray: Ray, depth: u32) -> Vec3 {
        if depth > self.settings.max_bounces {
            return skybox(ray.direction);
        }

        let Some(hit) = scene.world.intersect(ray, EPSILON, 200.0) else {
            return skybox(ray.direction);
        };
        let material = scene.materials[hit.material_id];
        let texel = material.texture.sample(hit.uv.0, hit.uv.1, hit.point);
        let surface_color = material.albedo * texel;
        let normal = mapped_normal(hit, material.normal_strength, material.texture);
        let view_direction = -ray.direction;

        let mut direct = surface_color * 0.055;
        for light in &scene.lights {
            let to_light = light.position - hit.point;
            let distance = to_light.length();
            let light_direction = to_light / distance;
            let visibility = self.light_visibility(
                scene,
                hit.point + normal * (EPSILON * 3.0),
                light_direction,
                distance,
            );
            if visibility <= 0.0 {
                continue;
            }

            let attenuation = light.intensity / (1.0 + 0.08 * distance + 0.025 * distance * distance);
            let diffuse = normal.dot(light_direction).max(0.0);
            let half_vector = (light_direction + view_direction).normalized();
            let specular = normal
                .dot(half_vector)
                .max(0.0)
                .powf(material.shininess)
                * material.specular;
            direct += (surface_color * diffuse + Vec3::ONE * specular)
                * light.color
                * attenuation
                * visibility;
        }

        let emitted = material.emission * texel;
        let cosine = (-ray.direction).dot(normal).clamp(0.0, 1.0);
        let base_fresnel = if material.refractive_index > 1.0 {
            ((material.refractive_index - 1.0) / (material.refractive_index + 1.0)).powi(2)
        } else {
            0.0
        };
        let fresnel = base_fresnel + (1.0 - base_fresnel) * (1.0 - cosine).powi(5);
        let mut reflection_weight =
            (material.reflectivity + material.transparency * fresnel).clamp(0.0, 1.0);
        let mut refraction_weight =
            (material.transparency * (1.0 - fresnel)).clamp(0.0, 1.0);

        let reflected = if reflection_weight > 0.001 {
            let direction = ray.direction.reflect(normal).normalized();
            self.trace(
                scene,
                Ray::new(hit.point + normal * (EPSILON * 4.0), direction),
                depth + 1,
            )
        } else {
            Vec3::ZERO
        };

        let refracted = if refraction_weight > 0.001 {
            let eta_ratio = if hit.front_face {
                1.0 / material.refractive_index
            } else {
                material.refractive_index
            };
            match refract(ray.direction, normal, eta_ratio) {
                Some(direction) => self.trace(
                    scene,
                    Ray::new(hit.point + direction * (EPSILON * 4.0), direction),
                    depth + 1,
                ),
                None => {
                    reflection_weight += refraction_weight;
                    refraction_weight = 0.0;
                    reflected
                }
            }
        } else {
            Vec3::ZERO
        };

        let local_weight = (1.0 - reflection_weight - refraction_weight).max(0.04);
        emitted
            + direct * local_weight
            + reflected * reflection_weight
            + refracted * refraction_weight
    }

    fn light_visibility(
        &self,
        scene: &Scene,
        mut origin: Vec3,
        direction: Vec3,
        mut remaining: f32,
    ) -> f32 {
        let mut visibility = 1.0;
        for _ in 0..8 {
            let ray = Ray::new(origin, direction);
            let Some(hit) = scene.world.intersect(ray, EPSILON, remaining - EPSILON) else {
                break;
            };
            let material = scene.materials[hit.material_id];
            if material.transparency <= 0.01 {
                return 0.0;
            }
            visibility *= material.transparency * 0.82;
            if visibility < 0.03 {
                return 0.0;
            }
            remaining -= hit.distance;
            origin = hit.point + direction * (EPSILON * 5.0);
        }
        visibility
    }
}

fn mapped_normal(
    hit: Hit,
    strength: f32,
    texture: crate::texture::TextureKind,
) -> Vec3 {
    let tangent_space = texture.tangent_normal(hit.uv.0, hit.uv.1, strength);
    (hit.tangent * tangent_space.x
        + hit.bitangent * tangent_space.y
        + hit.normal * tangent_space.z)
        .normalized()
}

/// Ley de Snell. `eta_ratio` es n1 / n2.
/// Retorna None cuando ocurre reflexion interna total.
pub fn refract(incident: Vec3, normal: Vec3, eta_ratio: f32) -> Option<Vec3> {
    let incident = incident.normalized();
    let normal = normal.normalized();
    let cos_theta = (-incident).dot(normal).min(1.0);
    let perpendicular = (incident + normal * cos_theta) * eta_ratio;
    let parallel_squared = 1.0 - perpendicular.length_squared();
    if parallel_squared < 0.0 {
        None
    } else {
        Some((perpendicular - normal * parallel_squared.sqrt()).normalized())
    }
}

fn skybox(direction: Vec3) -> Vec3 {
    let horizon = Vec3::new(0.20, 0.12, 0.30);
    let zenith = Vec3::new(0.012, 0.025, 0.11);
    let ground = Vec3::new(0.025, 0.018, 0.045);
    let mut color = if direction.y >= 0.0 {
        horizon.lerp(zenith, direction.y.clamp(0.0, 1.0).powf(0.52))
    } else {
        horizon.lerp(ground, (-direction.y).clamp(0.0, 1.0).powf(0.42))
    };

    let moon_direction = Vec3::new(-0.42, 0.78, -0.30).normalized();
    let moon_disc = direction.dot(moon_direction).max(0.0).powf(520.0);
    let moon_halo = direction.dot(moon_direction).max(0.0).powf(32.0);
    color += Vec3::new(0.72, 0.82, 1.0) * (moon_disc * 7.0 + moon_halo * 0.16);

    for star_direction in [
        Vec3::new(0.18, 0.91, -0.36),
        Vec3::new(0.61, 0.73, 0.31),
        Vec3::new(-0.70, 0.64, 0.18),
        Vec3::new(0.04, 0.82, 0.57),
        Vec3::new(-0.25, 0.94, 0.24),
    ] {
        let star = direction
            .dot(star_direction.normalized())
            .max(0.0)
            .powf(1800.0);
        color += Vec3::new(0.74, 0.82, 1.0) * star * 1.8;
    }
    color
}

fn to_rgba8(color: Vec3) -> [u8; 4] {
    let mapped = Vec3::new(aces(color.x), aces(color.y), aces(color.z));
    let gamma = Vec3::new(
        mapped.x.max(0.0).powf(1.0 / 2.2),
        mapped.y.max(0.0).powf(1.0 / 2.2),
        mapped.z.max(0.0).powf(1.0 / 2.2),
    )
    .clamp(0.0, 1.0);
    [
        (gamma.x * 255.0) as u8,
        (gamma.y * 255.0) as u8,
        (gamma.z * 255.0) as u8,
        255,
    ]
}

fn aces(value: f32) -> f32 {
    let value = value.max(0.0);
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    ((value * (a * value + b)) / (value * (c * value + d) + e)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_incidence_keeps_direction() {
        let incident = Vec3::new(0.0, -1.0, 0.0);
        let result = refract(incident, Vec3::new(0.0, 1.0, 0.0), 1.0 / 1.5)
            .expect("normal incidence must refract");
        assert!((result - incident).length() < 1.0e-5);
    }

    #[test]
    fn total_internal_reflection_returns_none() {
        let incident = Vec3::new(0.92, 0.39, 0.0).normalized();
        assert!(refract(incident, Vec3::new(0.0, -1.0, 0.0), 1.5).is_none());
    }
}