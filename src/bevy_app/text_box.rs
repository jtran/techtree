use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use bevy_mod_picking::prelude::*;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use smallvec::SmallVec;
use std::borrow::Cow;

use crate::{
    chart::{self, Flowchart},
    github::GithubIssueState,
};

use super::ui::UiState;

#[derive(Debug, Default, Resource)]
pub(crate) struct NodeIdEntityMap {
    pub map: bevy::utils::HashMap<chart::NodeId, Entity>,
}

impl NodeIdEntityMap {
    pub fn insert(&mut self, node_id: chart::NodeId, entity: Entity) {
        self.map.insert(node_id, entity);
    }

    pub fn get(&self, node_id: &chart::NodeId) -> Option<&Entity> {
        self.map.get(node_id)
    }
}

#[derive(Component)]
pub(crate) struct TextBox {
    pub node_id: chart::NodeId,
    state: GithubIssueState,
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

    pub fn is_state_open(&self) -> bool {
        match self.state {
            GithubIssueState::Open => true,
            GithubIssueState::Closed => false,
        }
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
    node_id_entity_map: &mut ResMut<NodeIdEntityMap>,
    text: &str,
    searchable_tokens: SmallVec<[String; 10]>,
    node_id: chart::NodeId,
    state: GithubIssueState,
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

    // Create a border mesh.
    let border_y = size.y / 10.0;
    let border_mesh =
        meshes.add(Cuboid::new(size.x + 0.004, border_y, cube_depth));

    let border_material = match state {
        GithubIssueState::Open => {
            materials.add(StandardMaterial {
                // Purple
                base_color: Color::rgb_u8(49, 114, 54),
                metallic: 0.1,
                perceptual_roughness: 0.5,
                ..Default::default()
            })
        }
        GithubIssueState::Closed => {
            materials.add(StandardMaterial {
                // Purple
                base_color: Color::rgb_u8(112, 72, 212),
                metallic: 0.1,
                perceptual_roughness: 0.5,
                ..Default::default()
            })
        }
    };

    let entity = commands
        .spawn(TextBox {
            node_id,
            state,
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
            // Border
            parent.spawn((
                PbrBundle {
                    mesh: border_mesh,
                    material: border_material,
                    transform: Transform::from_translation(Vec3::new(
                        // Offset everything by 0.001 to avoid z-fighting.
                        -0.001_f32,
                        (size.y - border_y) / 2.0 + 0.001,
                        0.001_f32,
                    )),
                    ..Default::default()
                },
                // Pass events through to the parent.
                Pickable::IGNORE,
            ));
            // Text
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: text_material,
                    transform: Transform::from_translation(Vec3::new(
                        text_mesh.bbox.size().x / -2_f32,
                        // Manual adjustment to center the text vertically.
                        // Determined experimentally.  Without the top-border, it
                        // was -0.25_f32.
                        -0.37_f32,
                        // Place the text in front of the cube.
                        cube_depth / 2_f32 + 0.1_f32,
                    )),
                    ..Default::default()
                },
                // Pass events through to the parent.
                Pickable::IGNORE,
            ));
        })
        .id();

    // Associate the two IDs.
    node_id_entity_map.insert(node_id, entity)
}

pub(crate) fn text_box_select_handler(
    mut events: EventReader<TextBoxSelectEvent>,
    query: Query<&mut TextBox>,
    mut state: ResMut<UiState>,
) {
    for event in events.read() {
        if let Ok(text_box) = query.get(event.entity) {
            state.select_node(text_box.node_id);
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

pub(crate) fn edge_drawing_system(
    mut gizmos: Gizmos,
    query: Query<(&TextBox, &Visibility, Entity)>,
    transform_query: Query<&GlobalTransform>,
    flowchart: Res<Flowchart>,
    node_id_entity_map: Res<NodeIdEntityMap>,
) {
    for (text_box, visibility, entity) in query.iter() {
        let node = flowchart.nodes_by_id.get(&text_box.node_id).unwrap();
        if matches!(visibility, Visibility::Hidden) {
            continue;
        }

        for node_id in node.depended_on_by_ids.iter() {
            let start = transform_query.get(entity).unwrap().translation();
            let other_node = flowchart.nodes_by_id.get(node_id).unwrap();
            let other_entity = *node_id_entity_map.get(&other_node.id).unwrap();
            let other_visibility = query.get(other_entity).unwrap().1;
            if matches!(other_visibility, Visibility::Hidden) {
                continue;
            }
            let end = transform_query
                .get(other_entity)
                .copied()
                .unwrap_or_default()
                .translation();
            let color = match text_box.state {
                GithubIssueState::Open => Color::rgb_u8(49, 114, 54),
                GithubIssueState::Closed => Color::rgb_u8(112, 72, 212),
            };
            gizmos.arrow(start, end, color);
        }
    }
}
