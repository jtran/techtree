use bevy::prelude::*;

use super::{text_box::TextBox, ui::NeedsLayoutEvent};

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
