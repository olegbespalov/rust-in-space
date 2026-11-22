use macroquad::prelude::*;

pub(super) const SOURCE_DIR: &str = "art/sprite-models";
const OVERHEAD: Vec3 = vec3(0.0, 0.0, 2.0);
const SCRAP_VIEW: Vec3 = vec3(0.0, 2.0, 1.8);
const CRATE_VIEW: Vec3 = vec3(-1.5, 0.85, 2.0);

pub(super) const SPRITES: [(&str, Vec3, u32); 19] = [
    ("enemy_projectile", OVERHEAD, 24),
    ("enemy_regular", vec3(0.0, 0.85, 2.0), 48),
    ("enemy_kamikaze", OVERHEAD, 40),
    ("enemy_guardian", OVERHEAD, 72),
    ("enemy_guardian_escalated", OVERHEAD, 72),
    ("enemy_interceptor", vec3(0.0, 0.55, 2.0), 40),
    // Show the heavy ship's recessed eye and three gun pods.
    ("enemy_heavy", vec3(0.0, 1.15, 2.0), 56),
    ("loot_scrap", SCRAP_VIEW, 48),
    ("loot_scrap_medium", SCRAP_VIEW, 48),
    ("loot_scrap_large", SCRAP_VIEW, 48),
    ("loot_gold", OVERHEAD, 48),
    ("loot_health", CRATE_VIEW, 48),
    ("loot_rapid_fire", CRATE_VIEW, 48),
    ("loot_big_bullet", CRATE_VIEW, 48),
    ("loot_shield", CRATE_VIEW, 48),
    ("magnetic_ammo", CRATE_VIEW, 48),
    ("invisibility", vec3(-0.68, 0.55, 2.0), 48),
    ("asteroid", vec3(0.0, 0.85, -2.0), 40),
    ("rare_asteroid", OVERHEAD, 40),
];

#[cfg(feature = "bake-sprites")]
pub(crate) fn run() {
    use super::{sprite_path, SPRITE_DIR};
    use crate::ship_model::load_mesh;
    use std::fs;

    fs::create_dir_all(SPRITE_DIR).expect("create sprite output directory");
    for (name, view, pixels) in SPRITES {
        let source = format!("{SOURCE_DIR}/{name}.glb");
        let bytes = fs::read(&source).unwrap_or_else(|error| panic!("Read {source}: {error}"));
        let mesh = load_mesh(&bytes).unwrap_or_else(|| panic!("Invalid model: {source}"));
        let target = render_target_ex(
            pixels,
            pixels,
            RenderTargetParams {
                depth: true,
                ..Default::default()
            },
        );
        push_camera_state();
        set_camera(&Camera3D {
            position: view,
            up: Vec3::Y,
            projection: Projection::Orthographics,
            fovy: 1.1,
            render_target: Some(target.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        draw_mesh(&mesh);
        pop_camera_state();

        // PNG export flips the render target into top-down image coordinates.
        let image = target.texture.get_texture_data();
        assert!(
            image.bytes.chunks_exact(4).any(|rgba| rgba[3] > 0),
            "Blank sprite: {name}"
        );
        image.export_png(&sprite_path(name));
    }
}
