use crate::{
    material::{
        AKUMA, BRICK, CAT_NOIR, GLASS, GOLD_LIGHT, LADYBUG, METAL, PARIS_STONE,
        ROOFTOP, WET_STREET,
    },
    math::Vec3,
    ray::Ray,
};

const EPSILON: f32 = 1.0e-4;

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub distance: f32,
    pub material_id: usize,
    pub uv: (f32, f32),
    pub front_face: bool,
}

#[derive(Debug, Clone)]
pub struct VoxelWorld {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    cells: Vec<u8>,
}

impl VoxelWorld {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
            cells: vec![0; width * height * depth],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, material_id: Option<usize>) {
        if !self.contains(x, y, z) {
            return;
        }
        let index = self.index(x as usize, y as usize, z as usize);
        self.cells[index] = material_id.map_or(0, |id| (id + 1) as u8);
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !self.contains(x, y, z) {
            return None;
        }
        let value = self.cells[self.index(x as usize, y as usize, z as usize)];
        if value == 0 {
            None
        } else {
            Some(value as usize - 1)
        }
    }

    pub fn occupied_count(&self) -> usize {
        self.cells.iter().filter(|cell| **cell != 0).count()
    }

    pub fn intersect(&self, ray: Ray, min_distance: f32, max_distance: f32) -> Option<Hit> {
        let (entry, exit, entry_normal) = self.bounds_interval(ray)?;
        let mut distance = entry.max(min_distance).max(0.0);
        if distance > exit || distance > max_distance {
            return None;
        }

        let origin_inside = self.point_inside(ray.origin);
        let start = ray.at(distance + EPSILON);
        let mut x = start.x.floor() as i32;
        let mut y = start.y.floor() as i32;
        let mut z = start.z.floor() as i32;
        let mut current = self.get(x, y, z);

        if !origin_inside && current.is_some() {
            return Some(make_hit(
                ray,
                distance,
                entry_normal,
                current.expect("material present"),
            ));
        }

        let step_x = axis_step(ray.direction.x);
        let step_y = axis_step(ray.direction.y);
        let step_z = axis_step(ray.direction.z);
        let mut next_x = next_boundary(ray.origin.x, ray.direction.x, x, step_x);
        let mut next_y = next_boundary(ray.origin.y, ray.direction.y, y, step_y);
        let mut next_z = next_boundary(ray.origin.z, ray.direction.z, z, step_z);
        let delta_x = axis_delta(ray.direction.x);
        let delta_y = axis_delta(ray.direction.y);
        let delta_z = axis_delta(ray.direction.z);

        for _ in 0..1024 {
            let (axis, next_distance) = if next_x <= next_y && next_x <= next_z {
                (0, next_x)
            } else if next_y <= next_z {
                (1, next_y)
            } else {
                (2, next_z)
            };

            distance = next_distance;
            if distance > exit + EPSILON || distance > max_distance {
                return None;
            }

            let (nx, ny, nz, step) = match axis {
                0 => (x + step_x, y, z, step_x),
                1 => (x, y + step_y, z, step_y),
                _ => (x, y, z + step_z, step_z),
            };
            let next_material = self.get(nx, ny, nz);

            if next_material != current {
                let axis_normal = Vec3::unit_axis(axis) * step as f32;
                let (material_id, outward_normal) = match (current, next_material) {
                    (None, Some(next_id)) => (next_id, -axis_normal),
                    (Some(current_id), None) => (current_id, axis_normal),
                    (Some(_), Some(next_id)) => (next_id, -axis_normal),
                    (None, None) => unreachable!(),
                };
                if distance >= min_distance {
                    return Some(make_hit(ray, distance, outward_normal, material_id));
                }
            }

            x = nx;
            y = ny;
            z = nz;
            current = next_material;
            match axis {
                0 => next_x += delta_x,
                1 => next_y += delta_y,
                _ => next_z += delta_z,
            }

            if !self.contains(x, y, z) && current.is_none() {
                return None;
            }
        }
        None
    }

    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (y * self.depth + z) * self.width + x
    }

    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && x < self.width as i32
            && y < self.height as i32
            && z < self.depth as i32
    }

    fn point_inside(&self, p: Vec3) -> bool {
        p.x >= 0.0
            && p.y >= 0.0
            && p.z >= 0.0
            && p.x < self.width as f32
            && p.y < self.height as f32
            && p.z < self.depth as f32
    }

    fn bounds_interval(&self, ray: Ray) -> Option<(f32, f32, Vec3)> {
        let max = Vec3::new(self.width as f32, self.height as f32, self.depth as f32);
        let mut near = f32::NEG_INFINITY;
        let mut far = f32::INFINITY;
        let mut entry_normal = Vec3::ZERO;

        for axis in 0..3 {
            let origin = ray.origin.component(axis);
            let direction = ray.direction.component(axis);
            let axis_max = max.component(axis);
            if direction.abs() < EPSILON {
                if origin < 0.0 || origin > axis_max {
                    return None;
                }
                continue;
            }

            let (axis_near, axis_far, normal) = if direction > 0.0 {
                ((-origin) / direction, (axis_max - origin) / direction, -Vec3::unit_axis(axis))
            } else {
                ((axis_max - origin) / direction, (-origin) / direction, Vec3::unit_axis(axis))
            };

            if axis_near > near {
                near = axis_near;
                entry_normal = normal;
            }
            far = far.min(axis_far);
            if near > far {
                return None;
            }
        }

        (far >= 0.0).then_some((near, far, entry_normal))
    }
}

fn make_hit(ray: Ray, distance: f32, outward_normal: Vec3, material_id: usize) -> Hit {
    let point = ray.at(distance);
    let front_face = ray.direction.dot(outward_normal) < 0.0;
    let normal = if front_face { outward_normal } else { -outward_normal };
    let uv = if outward_normal.x.abs() > 0.5 {
        (fract(point.z), 1.0 - fract(point.y))
    } else if outward_normal.y.abs() > 0.5 {
        (fract(point.x), fract(point.z))
    } else {
        (fract(point.x), 1.0 - fract(point.y))
    };

    let helper = if normal.y.abs() < 0.95 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = helper.cross(normal).normalized();
    let bitangent = normal.cross(tangent).normalized();

    Hit {
        point,
        normal,
        tangent,
        bitangent,
        distance,
        material_id,
        uv,
        front_face,
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

fn axis_step(direction: f32) -> i32 {
    if direction > 0.0 {
        1
    } else if direction < 0.0 {
        -1
    } else {
        0
    }
}

fn axis_delta(direction: f32) -> f32 {
    if direction.abs() < EPSILON {
        f32::INFINITY
    } else {
        1.0 / direction.abs()
    }
}

fn next_boundary(origin: f32, direction: f32, cell: i32, step: i32) -> f32 {
    if step == 0 {
        f32::INFINITY
    } else {
        let boundary = if step > 0 { cell + 1 } else { cell } as f32;
        (boundary - origin) / direction
    }
}

pub fn generate_diorama(seed: u32) -> VoxelWorld {
    const WIDTH: usize = 32;
    const HEIGHT: usize = 24;
    const DEPTH: usize = 32;
    let mut world = VoxelWorld::new(WIDTH, HEIGHT, DEPTH);

    // Ciudad procedural de Paris. Las calles forman una cuadricula y cada lote
    // recibe una altura determinista distinta a partir de la semilla.
    for x in 0..WIDTH as i32 {
        for z in 0..DEPTH as i32 {
            world.set(x, 0, z, Some(PARIS_STONE));
            let street = x.rem_euclid(8) <= 1 || z.rem_euclid(8) <= 1;
            if street {
                world.set(x, 1, z, Some(WET_STREET));
            } else {
                let lot_x = x.div_euclid(8);
                let lot_z = z.div_euclid(8);
                let variation = fbm(lot_x as f32 * 0.83, lot_z as f32 * 0.83, seed);
                let top = 3 + (variation * 4.8).floor() as i32;
                let facade = if (lot_x + lot_z) % 2 == 0 {
                    PARIS_STONE
                } else {
                    BRICK
                };
                for y in 1..top {
                    world.set(x, y, z, Some(facade));
                }
                world.set(x, top, z, Some(ROOFTOP));
            }
        }
    }

    // Azotea principal de 16x16: el escenario de la batalla.
    clear_box(&mut world, (8, 1, 7), (23, 23, 22));
    for x in 8..=23 {
        for z in 7..=22 {
            for y in 1..6 {
                let boundary = x == 8 || x == 23 || z == 7 || z == 22;
                world.set(x, y, z, Some(if boundary { BRICK } else { PARIS_STONE }));
            }
            world.set(x, 6, z, Some(ROOFTOP));
        }
    }

    // Parapeto de ladrillo; se dejan dos accesos para mejorar la silueta.
    for x in 8..=23 {
        if !(14..=17).contains(&x) {
            world.set(x, 7, 7, Some(BRICK));
            world.set(x, 7, 22, Some(BRICK));
        }
    }
    for z in 7..=22 {
        if !(13..=16).contains(&z) {
            world.set(8, 7, z, Some(BRICK));
            world.set(23, 7, z, Some(BRICK));
        }
    }

    // Charcos sobre el tejado para hacer evidente la reflexion del cielo y luces.
    for x in 9..=12 {
        for z in 17..=20 {
            world.set(x, 6, z, Some(WET_STREET));
        }
    }
    for x in 19..=21 {
        for z in 9..=11 {
            world.set(x, 6, z, Some(WET_STREET));
        }
    }

    // Claraboya de vidrio. El akuma que contiene permite ver la refraccion.
    clear_box(&mut world, (14, 7, 12), (18, 12, 16));
    for y in 7..=10 {
        for x in 14..=18 {
            for z in 12..=16 {
                if x == 14 || x == 18 || z == 12 || z == 16 {
                    world.set(x, y, z, Some(GLASS));
                }
            }
        }
    }
    fill_box(&mut world, (14, 11, 12), (18, 11, 16), GLASS);

    // Mariposa/akuma voxel: cuerpo central y alas dentro de la claraboya.
    for y in 8..=10 {
        world.set(16, y, 14, Some(AKUMA));
    }
    for &(x, y, z) in &[
        (15, 9, 14),
        (17, 9, 14),
        (14, 10, 14),
        (18, 10, 14),
        (15, 8, 14),
        (17, 8, 14),
    ] {
        world.set(x, y, z, Some(AKUMA));
    }

    // Ladybug voxel, con el material rojo de lunares.
    for &(x, y, z) in &[
        (10, 7, 12),
        (12, 7, 12),
        (10, 8, 12),
        (12, 8, 12),
        (10, 9, 12),
        (11, 9, 12),
        (12, 9, 12),
        (10, 10, 12),
        (11, 10, 12),
        (12, 10, 12),
        (9, 10, 12),
        (13, 10, 12),
        (10, 11, 12),
        (11, 11, 12),
        (12, 11, 12),
    ] {
        world.set(x, y, z, Some(LADYBUG));
    }

    // Yo-yo de Ladybug sobre el techo: disco rojo con centro negro.
    for x in 10..=12 {
        for z in 15..=17 {
            world.set(x, 7, z, Some(LADYBUG));
        }
    }
    world.set(11, 7, 16, Some(CAT_NOIR));

    // Cat Noir voxel, ojos/campana dorados y baston metalico.
    for &(x, y, z) in &[
        (20, 7, 18),
        (22, 7, 18),
        (20, 8, 18),
        (22, 8, 18),
        (20, 9, 18),
        (21, 9, 18),
        (22, 9, 18),
        (20, 10, 18),
        (21, 10, 18),
        (22, 10, 18),
        (19, 10, 18),
        (23, 10, 18),
        (20, 11, 18),
        (21, 11, 18),
        (22, 11, 18),
        (20, 12, 18),
        (22, 12, 18),
    ] {
        world.set(x, y, z, Some(CAT_NOIR));
    }
    world.set(21, 10, 17, Some(GOLD_LIGHT));
    for z in 12..=20 {
        world.set(23, 9, z, Some(METAL));
    }

    // Chimeneas de ladrillo y luces de la azotea.
    fill_box(&mut world, (9, 7, 8), (10, 10, 9), BRICK);
    fill_box(&mut world, (21, 7, 8), (22, 9, 9), BRICK);
    world.set(9, 11, 8, Some(GOLD_LIGHT));
    world.set(22, 10, 8, Some(GOLD_LIGHT));

    // Torre Eiffel estilizada al fondo. Se despeja el volumen para que su
    // silueta sea legible desde cualquier angulo de la camara.
    clear_box(&mut world, (2, 2, 23), (9, 23, 30));
    for y in 2..=6 {
        for &(x, z) in &[(3, 24), (7, 24), (3, 28), (7, 28)] {
            world.set(x, y, z, Some(METAL));
        }
    }
    for y in 7..=10 {
        for &(x, z) in &[(4, 25), (6, 25), (4, 27), (6, 27)] {
            world.set(x, y, z, Some(METAL));
        }
    }
    for x in 3..=7 {
        world.set(x, 6, 24, Some(METAL));
        world.set(x, 6, 28, Some(METAL));
    }
    for z in 24..=28 {
        world.set(3, 6, z, Some(METAL));
        world.set(7, 6, z, Some(METAL));
    }
    for x in 4..=6 {
        world.set(x, 10, 26, Some(METAL));
    }
    for z in 25..=27 {
        world.set(5, 10, z, Some(METAL));
    }
    for y in 11..=20 {
        world.set(5, y, 26, Some(METAL));
        if y % 3 == 0 {
            world.set(5, y, 25, Some(GOLD_LIGHT));
            world.set(5, y, 27, Some(GOLD_LIGHT));
        }
    }
    world.set(5, 21, 26, Some(GOLD_LIGHT));

    world
}

fn fill_box(
    world: &mut VoxelWorld,
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    material: usize,
) {
    for x in min.0..=max.0 {
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                world.set(x, y, z, Some(material));
            }
        }
    }
}

fn clear_box(world: &mut VoxelWorld, min: (i32, i32, i32), max: (i32, i32, i32)) {
    for x in min.0..=max.0 {
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                world.set(x, y, z, None);
            }
        }
    }
}

fn fbm(mut x: f32, mut z: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 0.55;
    let mut normalization = 0.0;
    for octave in 0..4 {
        sum += value_noise(x, z, seed.wrapping_add(octave * 1013)) * amplitude;
        normalization += amplitude;
        x *= 2.03;
        z *= 2.03;
        amplitude *= 0.5;
    }
    sum / normalization
}

fn value_noise(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let tx = smooth(x - x.floor());
    let tz = smooth(z - z.floor());
    let a = hash_grid(x0, z0, seed);
    let b = hash_grid(x0 + 1, z0, seed);
    let c = hash_grid(x0, z0 + 1, seed);
    let d = hash_grid(x0 + 1, z0 + 1, seed);
    lerp(lerp(a, b, tx), lerp(c, d, tx), tz)
}

fn hash_grid(x: i32, z: i32, seed: u32) -> f32 {
    let mut value = seed
        ^ (x as u32).wrapping_mul(0x9E37_79B9)
        ^ (z as u32).wrapping_mul(0x85EB_CA6B);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    (value & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dda_returns_the_nearest_voxel_surface() {
        let mut world = VoxelWorld::new(4, 4, 4);
        world.set(2, 1, 1, Some(PARIS_STONE));
        let ray = Ray::new(Vec3::new(-1.0, 1.5, 1.5), Vec3::new(1.0, 0.0, 0.0));
        let hit = world.intersect(ray, 0.001, 100.0).expect("expected hit");
        assert_eq!(hit.material_id, PARIS_STONE);
        assert!((hit.distance - 3.0).abs() < 0.001);
    }

    #[test]
    fn empty_voxel_returns_none_without_underflow() {
        let world = VoxelWorld::new(2, 2, 2);
        assert_eq!(world.get(0, 0, 0), None);
    }

    #[test]
    fn procedural_world_is_at_least_sixteen_by_sixteen() {
        let world = generate_diorama(42);
        assert!(world.width >= 16 && world.depth >= 16);
        assert!(world.occupied_count() > 16 * 16);
    }

    #[test]
    fn miraculous_landmarks_are_present() {
        let world = generate_diorama(2026);
        assert_eq!(world.get(16, 9, 14), Some(AKUMA));
        assert_eq!(world.get(14, 9, 12), Some(GLASS));
        assert_eq!(world.get(11, 10, 12), Some(LADYBUG));
        assert_eq!(world.get(21, 9, 18), Some(CAT_NOIR));
        assert_eq!(world.get(5, 18, 26), Some(METAL));
    }
}