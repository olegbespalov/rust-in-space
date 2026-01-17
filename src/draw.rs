use crate::components::*;
use macroquad::prelude::*;

pub fn draw_text_centered(text: &str, y_offset: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        screen_height() / 2.0 - dims.height / 2.0 + y_offset,
        size as f32,
        color,
    );
}

pub fn draw_ship(ship: &Ship) {
    let r = ship.rotation.to_radians();
    let v1 = ship.pos + vec2(r.cos(), r.sin()) * 20.0;
    let v2 = ship.pos + vec2((r + 2.5).cos(), (r + 2.5).sin()) * 15.0;
    let v3 = ship.pos + vec2((r - 2.5).cos(), (r - 2.5).sin()) * 15.0;
    let color = if ship.rapid_fire_timer > 0.0 {
        PURPLE
    } else {
        WHITE
    };
    draw_triangle(v1, v2, v3, color);
    draw_triangle_lines(v1, v2, v3, 2.0, BLUE);
}
