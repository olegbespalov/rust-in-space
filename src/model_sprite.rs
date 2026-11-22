use macroquad::prelude::*;

#[cfg(any(test, feature = "bake-sprites"))]
mod bake;
#[cfg(feature = "bake-sprites")]
pub(crate) use bake::run as bake_sprites;

const SPRITE_DIR: &str = "assets/sprites";

fn sprite_path(name: &str) -> String {
    format!("{SPRITE_DIR}/{name}.png")
}

pub struct ModelSprite(Texture2D);

impl ModelSprite {
    pub async fn load(name: &str, fallback: &str) -> Self {
        let path = sprite_path(name);
        let texture = match load_image(&path).await {
            Ok(image) => Texture2D::from_image(&image),
            Err(error) => {
                eprintln!("Could not load {path}: {error}. Using the fallback sprite.");
                load_texture(fallback).await.unwrap()
            }
        };
        texture.set_filter(FilterMode::Nearest);
        Self(texture)
    }

    pub fn draw(&self, position: Vec2, size: f32, rotation: f32) {
        self.draw_tinted(position, size, rotation, WHITE);
    }

    pub fn draw_tinted(&self, position: Vec2, size: f32, rotation: f32, tint: Color) {
        draw_texture_ex(
            &self.0,
            position.x - size / 2.0,
            position.y - size / 2.0,
            tint,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                rotation,
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::bake::{SOURCE_DIR, SPRITES};
    use super::sprite_path;
    use crate::ship_model::MODEL_MESH_INDEX_CAPACITY;
    use gltf::{image, mesh, Gltf};
    use macroquad::texture::Image;
    use std::fs;

    #[test]
    fn model_assets_match_the_mesh_loader() {
        let player_paths = [
            "player_ship",
            "player_delta",
            "player_specter",
            "player_vector",
        ]
        .map(|name| format!("assets/models/{name}.glb"));
        let sprite_paths = SPRITES.map(|(name, _, _)| format!("{SOURCE_DIR}/{name}.glb"));
        for path in player_paths.into_iter().chain(sprite_paths) {
            let bytes = fs::read(&path).unwrap_or_else(|error| panic!("Read {path}: {error}"));
            let glb = Gltf::from_slice(&bytes).unwrap();
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

    #[test]
    fn baked_sprites_have_the_expected_dimensions_and_transparency() {
        for (name, _, pixels) in SPRITES {
            let path = sprite_path(name);
            let bytes = fs::read(&path).unwrap_or_else(|error| panic!("Read {path}: {error}"));
            let image = Image::from_file_with_format(&bytes, None).unwrap();
            assert_eq!(
                (u32::from(image.width), u32::from(image.height)),
                (pixels, pixels)
            );
            assert!(
                image.bytes.chunks_exact(4).any(|rgba| rgba[3] > 0),
                "Blank sprite: {name}"
            );
            assert!(
                image.bytes.chunks_exact(4).any(|rgba| rgba[3] == 0),
                "Opaque sprite: {name}"
            );
        }
    }
}
