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

        let mut direct = surface_color * 0.10;
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
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let horizon = Vec3::new(0.82, 0.62, 0.48);
    let zenith = Vec3::new(0.08, 0.23, 0.52);
    let ground = Vec3::new(0.05, 0.07, 0.09);
    let mut color = if direction.y >= 0.0 {
        horizon.lerp(zenith, t.powf(0.65))
    } else {
        horizon.lerp(ground, (-direction.y).min(1.0))
    };

    let sun_direction = Vec3::new(-0.35, 0.80, -0.28).normalized();
    let sun = direction.dot(sun_direction).max(0.0).powf(420.0);
    color += Vec3::new(1.0, 0.78, 0.46) * (sun * 8.0);
    color
}

fn to_rgba8(color: Vec3) -> [u8; 4] {
    let mapped = Vec3::new(
        color.x / (1.0 + color.x),
        color.y / (1.0 + color.y),
        color.z / (1.0 + color.z),
    );
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

