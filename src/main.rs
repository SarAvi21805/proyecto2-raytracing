use proyecto2_raytracing::{
    camera::Camera,
    math::Vec3,
    renderer::Renderer,
    scene::Scene,
};
use raylib::prelude::*;
use std::time::Instant;

const RENDER_WIDTH: usize = 320;
const RENDER_HEIGHT: usize = 180;
const WINDOW_SCALE: i32 = 3;

fn main() {
    let window_width = RENDER_WIDTH as i32 * WINDOW_SCALE;
    let window_height = RENDER_HEIGHT as i32 * WINDOW_SCALE;
    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Proyecto 2 - Raytracing CPU")
        .build();
    rl.set_target_fps(60);

    let image = Image::gen_image_color(
        RENDER_WIDTH as i32,
        RENDER_HEIGHT as i32,
        Color::BLACK,
    );
    let mut screen_texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("No se pudo crear la textura de presentacion");
    screen_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    let renderer = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
    let mut scene = Scene::diorama(2026);
    let mut seed = 2026_u32;
    let mut camera = Camera::new(Vec3::new(12.0, 6.0, 12.0));
    let mut dirty = true;
    let mut render_ms = 0_u128;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let angular_speed = 1.15 * dt;

        if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
            camera.orbit(-angular_speed, 0.0);
            dirty = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
            camera.orbit(angular_speed, 0.0);
            dirty = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
            camera.orbit(0.0, angular_speed);
            dirty = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
            camera.orbit(0.0, -angular_speed);
            dirty = true;
        }

        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > f32::EPSILON {
            camera.zoom(-wheel * 1.8);
            dirty = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            camera.zoom(-2.0);
            dirty = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_E) {
            camera.zoom(2.0);
            dirty = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            seed = seed.wrapping_add(1);
            scene = Scene::diorama(seed);
            dirty = true;
        }

        if dirty {
            let start = Instant::now();
            let pixels = renderer.render(&scene, camera);
            screen_texture
                .update_texture(&pixels)
                .expect("No se pudo actualizar el framebuffer");
            render_ms = start.elapsed().as_millis();
            dirty = false;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &screen_texture,
            Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
            Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        d.draw_rectangle(8, 8, 440, 52, Color::new(0, 0, 0, 170));
        d.draw_text(
            "Rotar: WASD/flechas | Zoom: rueda o Q/E | Nuevo terreno: R",
            16,
            15,
            16,
            Color::RAYWHITE,
        );
        d.draw_text(
            &format!(
                "CPU: {} hilos | ultimo render: {} ms | seed: {}",
                renderer.settings.threads, render_ms, seed
            ),
            16,
            37,
            16,
            Color::GOLD,
        );
    }
}
