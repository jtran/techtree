use bevy::prelude::*;

use crate::chart::Flowchart;

use super::{
    text_box::{NodeIdEntityMap, TextBox},
    ui::NeedsLayoutEvent,
};

pub(crate) fn relayout_handler(
    mut events: EventReader<NeedsLayoutEvent>,
    mut transform_query: Query<(&mut TextBox, &Visibility, Entity)>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    let mut visible_entities = Vec::new();
    for (_, visibility, entity) in transform_query.iter() {
        match visibility {
            Visibility::Hidden => {}
            Visibility::Inherited | Visibility::Visible => {
                visible_entities.push(entity);
            }
        }
    }

    let num_nodes = visible_entities.len();

    if num_nodes == 0 {
        // Everything is filtered out.  Nothing to do.
        return;
    }

    let num_rows = (num_nodes as f32 * 4.0).sqrt().ceil() as usize;
    let half_rows = num_rows as f32 / 2_f32;
    let num_cols = num_nodes.div_ceil(num_rows);
    let half_cols = num_cols as f32 / 2_f32;
    let x_axis_flip = 1_f32;
    let y_axis_flip = -1_f32;

    let mut i = 0_usize;
    let mut j = 0_usize;
    for entity in visible_entities {
        let i_f32 = i as f32;
        let j_f32 = j as f32;

        let width = 20_f32;
        let height = 2_f32;

        let margin_x = 1_f32;
        let margin_y = 1_f32;

        let x_offset = x_axis_flip
            * ((width + margin_x) * j_f32 - (width + margin_x) * half_cols);
        let y_offset = y_axis_flip
            * ((height + margin_y) * i_f32 - (height + margin_y) * half_rows);
        let translation = Vec3::new(x_offset, y_offset, 0_f32);

        let (mut text_box, _, _) = transform_query
            .get_mut(entity)
            .expect("entity should exist");
        text_box.target_translation = translation;

        i += 1;
        if i >= num_rows {
            i = 0;
            j += 1;
        }
    }
}

pub(crate) fn animation_system(
    mut transform_query: Query<(&mut Transform, &TextBox)>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    let speed = 5_f32;

    for (mut transform, text_box) in transform_query.iter_mut() {
        transform.translation = transform
            .translation
            .lerp(text_box.target_translation, 1.0 - (-speed * dt).exp());
    }
}

pub(crate) fn force_system(
    global_transform_query: Query<&GlobalTransform>,
    mut velocity_query: Query<(
        &mut TextBox,
        &mut Transform,
        &Visibility,
        Entity,
    )>,
    visibility_query: Query<&Visibility>,
    flowchart: Res<Flowchart>,
    node_id_entity_map: Res<NodeIdEntityMap>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let k = if keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight)
    {
        500_f32
    } else {
        1000_f32
    };
    let k_edge = 0.00001_f32;

    let dt = time.delta().as_secs_f32();

    for (text_box, mut transform, visibility, entity) in
        velocity_query.iter_mut()
    {
        let node = flowchart.nodes_by_id.get(&text_box.node_id).unwrap();
        if matches!(visibility, Visibility::Hidden) {
            continue;
        }

        let global_transform = global_transform_query
            .get(entity)
            .expect("entity should exist");

        let mut force = Vec3::ZERO;

        for (other_node_id, other_entity) in node_id_entity_map.iter() {
            // Ignore self.
            if *other_node_id == node.id {
                continue;
            }
            let other_visibility = visibility_query
                .get(*other_entity)
                .expect("entity should exist");
            // Ignore hidden nodes.
            if matches!(other_visibility, Visibility::Hidden) {
                continue;
            }

            let other_transform = global_transform_query
                .get(*other_entity)
                .expect("entity should exist");
            // Repel.  The force is towards the current node.
            let direction =
                global_transform.translation() - other_transform.translation();
            let distance = direction.length();
            let force_magnitude = 1.0 / distance.powi(2);
            force += direction.normalize() * force_magnitude;
        }

        let force_from_nodes = force;

        // Edges.
        for other_node_id in node.depends_on_ids.iter() {
            let other_entity = *node_id_entity_map.get(other_node_id).unwrap();
            let other_visibility = visibility_query
                .get(other_entity)
                .expect("entity should exist");
            // Ignore hidden nodes.
            if matches!(other_visibility, Visibility::Hidden) {
                continue;
            }

            let other_transform = global_transform_query
                .get(other_entity)
                .expect("entity should exist");
            // The direction is towards the current node.
            let direction =
                global_transform.translation() - other_transform.translation();
            let distance = direction.length();
            let uncompressed_length = 20_f32;
            let dx = distance - uncompressed_length;
            let force_magnitude = (-k_edge * dx).min(0.0);
            if force_magnitude.is_nan() {
                // eprintln!("distance is NaN: direction={direction:?} other_transform.translation()={:?}", other_transform.translation());
            } else {
                force += direction.normalize() * force_magnitude;
                eprintln!(
                    "computing edge force: direction.normalize()={:?}, distance={distance}, dx={dx}, force_magnitude={force_magnitude}",
                    direction.normalize(),
                );
            }
        }

        let force_from_edges = force - force_from_nodes;
        if force_from_edges.length().abs() > 0.1 {
            eprintln!("force_from_edges: {}", force_from_edges.length());
        }

        transform.translation += k * force * dt;
    }
}
