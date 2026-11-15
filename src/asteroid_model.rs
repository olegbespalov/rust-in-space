use crate::ship_model::load_mesh;
use macroquad::prelude::*;

const RENDER_SIZE: u32 = 192;

pub struct AsteroidModel {
    target: RenderTarget,
}

#[cfg(test)]
mod tests {
    use crate::ship_model::MODEL_MESH_INDEX_CAPACITY;
    use gltf::{image, mesh, Gltf};
    use macroquad::texture::Image;

    #[test]
    fn asteroid_assets_match_the_mesh_loader() {
        for bytes in [
            include_bytes!("../assets/models/asteroid.glb").as_slice(),
            include_bytes!("../assets/models/rare_asteroid.glb").as_slice(),
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

impl AsteroidModel {
    pub async fn load(path: &str) -> Option<Self> {
        let mesh = match load_file(path).await {
            Ok(bytes) => load_mesh(&bytes),
            Err(_) => None,
        };
        let Some(mesh) = mesh else {
            eprintln!("Could not load {path}. Using the asteroid sprite.");
            return None;
        };

        let target = render_target_ex(
            RENDER_SIZE,
            RENDER_SIZE,
            RenderTargetParams {
                depth: true,
                ..Default::default()
            },
        );
        push_camera_state();
        set_camera(&Camera3D {
            position: vec3(0.0, 0.0, 2.0),
            up: Vec3::Y,
            projection: Projection::Orthographics,
            fovy: 1.1,
            render_target: Some(target.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        draw_mesh(&mesh);
        pop_camera_state();

        Some(Self { target })
    }

    pub fn draw(&self, position: Vec2, size: f32) {
        draw_texture_ex(
            &self.target.texture,
            position.x - size / 2.0,
            position.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}
