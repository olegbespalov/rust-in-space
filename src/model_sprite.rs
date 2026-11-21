use crate::ship_model::load_mesh;
use macroquad::prelude::*;

pub struct ModelSprite {
    fallback: Texture2D,
    target: Option<RenderTarget>,
}

#[cfg(test)]
mod tests {
    use crate::ship_model::MODEL_MESH_INDEX_CAPACITY;
    use gltf::{image, mesh, Gltf};
    use macroquad::texture::Image;

    #[test]
    fn model_assets_match_the_mesh_loader() {
        for bytes in [
            include_bytes!("../assets/models/player_ship.glb").as_slice(),
            include_bytes!("../assets/models/player_delta.glb").as_slice(),
            include_bytes!("../assets/models/asteroid.glb").as_slice(),
            include_bytes!("../assets/models/rare_asteroid.glb").as_slice(),
            include_bytes!("../assets/models/invisibility.glb").as_slice(),
            include_bytes!("../assets/models/enemy_regular.glb").as_slice(),
            include_bytes!("../assets/models/enemy_kamikaze.glb").as_slice(),
            include_bytes!("../assets/models/enemy_guardian.glb").as_slice(),
            include_bytes!("../assets/models/enemy_projectile.glb").as_slice(),
            include_bytes!("../assets/models/enemy_interceptor.glb").as_slice(),
            include_bytes!("../assets/models/enemy_heavy.glb").as_slice(),
            include_bytes!("../assets/models/loot_scrap.glb").as_slice(),
            include_bytes!("../assets/models/loot_gold.glb").as_slice(),
            include_bytes!("../assets/models/loot_health.glb").as_slice(),
            include_bytes!("../assets/models/loot_rapid_fire.glb").as_slice(),
            include_bytes!("../assets/models/loot_big_bullet.glb").as_slice(),
            include_bytes!("../assets/models/loot_shield.glb").as_slice(),
            include_bytes!("../assets/models/magnetic_ammo.glb").as_slice(),
        ] {
            let glb = Gltf::from_slice(bytes).unwrap();
            let blob = glb.blob.as_deref().unwrap();
            let mut meshes = glb.meshes();
            let model = meshes.next().unwrap();
            assert!(meshes.next().is_none());
            let mut primitives = model.primitives();
            let primitive = primitives.next().unwrap();
            assert!(primitives.next().is_none());
            assert_eq!(primitive.mode(), mesh::Mode::Triangles);
            let reader = primitive.reader(|_| Some(blob));
            let positions = reader.read_positions().unwrap();
            assert!(positions.len() < 10_000);
            assert_eq!(reader.read_normals().unwrap().len(), positions.len());
            assert_eq!(
                reader.read_tex_coords(0).unwrap().into_f32().len(),
                positions.len()
            );
            assert!(reader.read_indices().unwrap().into_u32().len() < MODEL_MESH_INDEX_CAPACITY);
            let texture = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .unwrap()
                .texture();
            let image::Source::View { view, .. } = texture.source().source() else {
                panic!("Expected an embedded texture");
            };
            let image = &blob[view.offset()..view.offset() + view.length()];
            assert!(Image::from_file_with_format(image, None).is_ok());
        }
    }
}

impl ModelSprite {
    pub async fn load(sprite_path: &str, model_path: &str, view: Vec3, pixels: u32) -> Self {
        let fallback = load_texture(sprite_path).await.unwrap();
        fallback.set_filter(FilterMode::Nearest);
        let target = Self::bake(model_path, view, pixels).await;
        Self { fallback, target }
    }

    pub async fn with_model(&self, path: &str, view: Vec3, pixels: u32) -> Self {
        let target = Self::bake(path, view, pixels).await;
        Self {
            fallback: self.fallback.clone(),
            target,
        }
    }

    async fn bake(path: &str, view_position: Vec3, pixels: u32) -> Option<RenderTarget> {
        let mesh = match load_file(path).await {
            Ok(bytes) => load_mesh(&bytes),
            Err(_) => None,
        };
        let Some(mesh) = mesh else {
            eprintln!("Could not load {path}. Using the fallback sprite.");
            return None;
        };

        let target = render_target_ex(
            pixels,
            pixels,
            RenderTargetParams {
                depth: true,
                ..Default::default()
            },
        );
        target.texture.set_filter(FilterMode::Nearest);
        push_camera_state();
        set_camera(&Camera3D {
            position: view_position,
            up: Vec3::Y,
            projection: Projection::Orthographics,
            fovy: 1.1,
            render_target: Some(target.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        draw_mesh(&mesh);
        pop_camera_state();

        Some(target)
    }

    pub fn draw(&self, position: Vec2, size: f32, rotation: f32) {
        let (texture, flip_y) = match &self.target {
            Some(target) => (&target.texture, true),
            None => (&self.fallback, false),
        };
        draw_texture_ex(
            texture,
            position.x - size / 2.0,
            position.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                rotation,
                flip_y,
                ..Default::default()
            },
        );
    }
}
