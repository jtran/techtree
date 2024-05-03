use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use bevy_mod_picking::prelude::*;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use smallvec::SmallVec;
use std::borrow::Cow;

use crate::chart;

use super::ui::UiState;

#[derive(Default, Component)]
pub(crate) struct TextBox {
    pub node_id: chart::NodeId,
    pub target_translation: Vec3,
    searchable_tokens: SmallVec<[String; 10]>,
}

impl TextBox {
    pub fn matches(&self, filter: &str) -> bool {
        filter.split_whitespace().all(|key| {
            self.searchable_tokens
                .iter()
                .any(|token| token.contains(key))
        })
    }
}

#[derive(Event)]
pub(crate) struct TextBoxSelectEvent {
    entity: Entity,
}

#[derive(Event)]
pub(crate) struct TextBoxDeselectEvent {
    entity: Entity,
}

impl From<ListenerInput<Pointer<Select>>> for TextBoxSelectEvent {
    fn from(value: ListenerInput<Pointer<Select>>) -> Self {
        TextBoxSelectEvent {
            entity: value.target,
        }
    }
}

impl From<ListenerInput<Pointer<Deselect>>> for TextBoxDeselectEvent {
    fn from(value: ListenerInput<Pointer<Deselect>>) -> Self {
        TextBoxDeselectEvent {
            entity: value.target,
        }
    }
}

const ELLIPSIS: &str = "…";

// Spawn a box with text.  Pass in the mesh generator so that we don't need to
// parse the font every time.
pub(crate) fn spawn(
    commands: &mut Commands,
    mesh_generator: &mut MeshGenerator<Face>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    text: &str,
    searchable_tokens: SmallVec<[String; 10]>,
    node_id: chart::NodeId,
    size: Vec2,
    translation: Vec3,
) {
    let num_chars = ((size.x * 1.9_f32).floor() as usize).max(1) - 1;
    let text = if text.len() <= num_chars {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.chars().take(num_chars).collect::<String>() + ELLIPSIS)
    };

    // Generate the text mesh.
    let mesh_transform =
        Mat4::from_scale(Vec3::new(1_f32, 1_f32, 0.2_f32)).to_cols_array();

    let text_mesh: MeshText = mesh_generator
        .generate_section(text.as_ref(), false, Some(&mesh_transform))
        .unwrap();

    let vertices = text_mesh.vertices;
    let positions: Vec<[f32; 3]> = vertices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();
    let uvs = vec![[0_f32, 0_f32]; positions.len()];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.compute_flat_normals();

    // Material for the text.
    let text_material = materials.add(StandardMaterial {
        base_color: Color::rgb_u8(50, 50, 255),
        metallic: 0.3,
        perceptual_roughness: 0.1,
        ..Default::default()
    });

    // Create a cube mesh.
    let cube_depth = 1_f32;
    let cube_mesh = meshes.add(Cuboid::new(size.x, size.y, cube_depth));

    // Material for the cube.
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::rgb_u8(240, 240, 240),
        metallic: 0.1,
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    commands
        .spawn(TextBox {
            node_id,
            // Initial target should be the same as the transform's translation.
            target_translation: translation,
            searchable_tokens,
        })
        .insert((
            PickableBundle::default(),
            On::<Pointer<Select>>::send_event::<TextBoxSelectEvent>(),
            On::<Pointer<Deselect>>::send_event::<TextBoxDeselectEvent>(),
        ))
        .insert(PbrBundle {
            mesh: cube_mesh,
            material: cube_material,
            transform: Transform::from_translation(translation),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: text_material,
                    transform: Transform::from_translation(Vec3::new(
                        text_mesh.bbox.size().x / -2_f32,
                        // Manual adjustment to center the text vertically.
                        // Determined experimentally.
                        -0.25_f32,
                        // Place the text in front of the cube.
                        cube_depth / 2_f32 + 0.1_f32,
                    )),
                    ..Default::default()
                },
                // Pass events through to the parent.
                Pickable::IGNORE,
            ));
        });
}

pub(crate) fn text_box_select_handler(
    mut events: EventReader<TextBoxSelectEvent>,
    query: Query<&mut TextBox>,
    mut state: ResMut<UiState>,
) {
    for event in events.read() {
        if let Ok(text_box) = query.get(event.entity) {
            state.select_node(text_box.node_id.clone());
        }
    }
}

pub(crate) fn text_box_deselect_handler(
    mut events: EventReader<TextBoxDeselectEvent>,
    query: Query<&mut TextBox>,
    mut state: ResMut<UiState>,
) {
    for event in events.read() {
        if let Ok(text_box) = query.get(event.entity) {
            state.deselect_node(&text_box.node_id);
        }
    }
}
