use gltf::{buffer, image, mesh, Gltf};
use macroquad::prelude::*;

pub const SHIP_MESH_INDEX_CAPACITY: usize = 12_001;
const MODEL_PATH: &str = "assets/models/player_ship.glb";
const RENDER_SIZE: u32 = 192;

pub struct ShipModel {
    mesh: Mesh,
    target: RenderTarget,
}

impl ShipModel {
    pub async fn load() -> Option<Self> {
        let mesh = match load_file(MODEL_PATH).await {
            Ok(bytes) => load_mesh(&bytes),
            Err(_) => None,
        };
        if mesh.is_none() {
            eprintln!("Could not load {MODEL_PATH}. Using the player ship sprite.");
        }

        mesh.map(|mesh| Self {
            mesh,
            target: render_target_ex(
                RENDER_SIZE,
                RENDER_SIZE,
                RenderTargetParams {
                    depth: true,
                    ..Default::default()
                },
            ),
        })
    }

    pub fn draw(&self, position: Vec2, size: f32, rotation: f32, thrust: f32) {
        push_camera_state();
        set_camera(&Camera3D {
            position: vec3(0.0, -0.27 - 0.36 * thrust, 1.5),
            up: Vec3::Y,
            projection: Projection::Orthographics,
            fovy: 1.04,
            render_target: Some(self.target.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        draw_mesh(&self.mesh);
        pop_camera_state();

        draw_texture_ex(
            &self.target.texture,
            position.x - size / 2.0,
            position.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                rotation,
                // The 3D render target uses a bottom-left origin.
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

// The game ships a single textured, triangulated mesh with its transform baked in.
fn load_mesh(bytes: &[u8]) -> Option<Mesh> {
    let glb = Gltf::from_slice(bytes).ok()?;
    let blob = glb.blob.as_deref()?;
    let mut meshes = glb.meshes();
    let ship_mesh = meshes.next()?;
    let mut primitives = ship_mesh.primitives();
    let primitive = primitives.next()?;
    if meshes.next().is_some()
        || primitives.next().is_some()
        || primitive.mode() != mesh::Mode::Triangles
    {
        return None;
    }

    let reader = primitive.reader(|buffer| match buffer.source() {
        buffer::Source::Bin => Some(blob),
        buffer::Source::Uri(_) => None,
    });
    let positions = reader.read_positions()?;
    let normals = reader.read_normals()?;
    let uv = reader.read_tex_coords(0)?.into_f32();
    let indices = reader.read_indices()?.into_u32();
    if positions.len() >= 10_000
        || normals.len() != positions.len()
        || uv.len() != positions.len()
        || indices.len() >= SHIP_MESH_INDEX_CAPACITY
    {
        return None;
    }

    let bounds = primitive.bounding_box();
    let min = Vec3::from_array(bounds.min);
    let max = Vec3::from_array(bounds.max);
    let center = (min + max) / 2.0;
    let span = (max - min).truncate().max_element();
    if !span.is_finite() || span <= 0.0 {
        return None;
    }

    let light = vec3(-0.45, 0.65, 1.0).normalize();
    let vertices: Vec<_> = positions
        .zip(normals)
        .zip(uv)
        .map(|((position, normal), uv)| {
            let position = (Vec3::from_array(position) - center) / span;
            let diffuse = Vec3::from_array(normal).dot(light).max(0.0);
            let brightness = 0.65 + 0.35 * diffuse;
            Vertex::new2(
                position,
                Vec2::from_array(uv),
                Color::new(brightness, brightness, brightness, 1.0),
            )
        })
        .collect();
    let indices = indices
        .map(|index| (index < vertices.len() as u32).then_some(index as u16))
        .collect::<Option<Vec<_>>>()?;

    let material = primitive.material().pbr_metallic_roughness();
    let texture = material.base_color_texture()?.texture();
    let image::Source::View { view, .. } = texture.source().source() else {
        return None;
    };
    if !matches!(view.buffer().source(), buffer::Source::Bin) {
        return None;
    }
    let image_bytes = blob.get(view.offset()..view.offset().checked_add(view.length())?)?;
    let image = Image::from_file_with_format(image_bytes, None).ok()?;

    Some(Mesh {
        vertices,
        indices,
        texture: Some(Texture2D::from_image(&image)),
    })
}
