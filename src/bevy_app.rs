// cSpell: ignore bbox consts Fira

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use smallvec::SmallVec;

use crate::chart::Flowchart;
use crate::{
    facade::{self, DepsArgs},
    AppResult,
};

mod input;
mod layout;
mod text_box;
mod ui;

pub(crate) fn main(args: crate::GuiArgs) -> AppResult<()> {
    let mut flowchart = facade::build_dependencies(DepsArgs {
        title: None,
        all: args.all,
        issues: args.issues,
        include_project: None,
        prior_days: None,
    })?;
    // Remove nodes that don't match the filter.
    flowchart.prune();

    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(1_f32, 1_f32, 1_f32)))
        .insert_resource(flowchart)
        .insert_resource(selection::SelectionPluginSettings {
            is_enabled: true,
            click_nothing_deselect_all: true,
            // Turn off multi-select.
            use_multiselect_default_inputs: false,
        })
        .add_event::<text_box::TextBoxSelectEvent>()
        .add_event::<text_box::TextBoxDeselectEvent>()
        .add_event::<ui::NeedsLayoutEvent>()
        .add_event::<ui::FilterChangeEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .init_resource::<ui::UiState>()
        .add_plugins(bevy_egui::EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, ui::immediate_system)
        .add_systems(Update, ui::filter_events)
        .add_systems(Update, input::keyboard_system)
        .add_systems(Update, input::events_system)
        .add_systems(Update, layout::relayout_handler)
        .add_systems(Update, layout::animation_system)
        .add_systems(Update, text_box::text_box_select_handler)
        .add_systems(Update, text_box::text_box_deselect_handler)
        .run();

    Ok(())
}

/// Marker for the scene camera.
#[derive(Default, Component)]
struct SceneCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    flowchart: Res<Flowchart>,
) {
    let font_bytes =
        include_bytes!("../assets/fonts/Fira_Code_v6.2/FiraCode-Regular.ttf");
    let mut mesh_generator = meshtext::MeshGenerator::new(font_bytes);

    let num_nodes = flowchart.num_nodes();

    let num_rows = (num_nodes as f32 * 4.0).sqrt().ceil() as usize;
    let half_rows = num_rows as f32 / 2_f32;
    let num_cols = num_nodes.div_ceil(num_rows);
    let half_cols = num_cols as f32 / 2_f32;
    let x_axis_flip = 1_f32;
    let y_axis_flip = -1_f32;

    let mut i = 0_usize;
    let mut j = 0_usize;
    for index in 0..num_nodes {
        let i_f32 = i as f32;
        let j_f32 = j as f32;

        let node = flowchart
            .get_node_by_index(index)
            .expect("index out of bounds for flowchart nodes");

        let width = 20_f32;
        let height = 2_f32;

        let margin_x = 1_f32;
        let margin_y = 1_f32;

        let x_offset = x_axis_flip
            * ((width + margin_x) * j_f32 - (width + margin_x) * half_cols);
        let y_offset = y_axis_flip
            * ((height + margin_y) * i_f32 - (height + margin_y) * half_rows);
        let translation = Vec3::new(x_offset, y_offset, 0_f32);
        let size = Vec2::new(width, height);
        let text = node.text.clone();
        let mut searchable_tokens = text
            .to_lowercase()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<SmallVec<_>>();
        for label in &node.labels {
            searchable_tokens.push(label.clone());
        }
        text_box::spawn(
            &mut commands,
            &mut mesh_generator,
            &mut meshes,
            &mut materials,
            text.as_str(),
            // TODO: Use a Copy type.
            searchable_tokens,
            node.id.clone(),
            size,
            translation,
        );

        i += 1;
        if i >= num_rows {
            i = 0;
            j += 1;
        }
    }

    // Lighting.
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(
            -std::f32::consts::FRAC_PI_4,
        )),
        ..Default::default()
    });
    // Camera.
    commands
        .spawn(Camera3dBundle {
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::WindowSize(
                    32_f32,
                ),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0_f32, 0_f32, 6_f32)
                .looking_at(Vec3::new(0_f32, 0_f32, 0f32), Vec3::Y),
            ..Default::default()
        })
        .insert(SceneCamera {});
}
