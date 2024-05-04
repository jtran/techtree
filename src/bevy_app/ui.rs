use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::chart;
use crate::chart::NodeId;

use super::text_box::TextBox;
use super::ORTHOGRAPHIC_PROJECTION;

#[derive(Debug, Resource)]
pub(crate) struct UiState {
    filter_text: String,
    show_closed: bool,
    pub camera_scale: f32,
    selected_node_id: Option<NodeId>,
    input_debounce_timer: Timer,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            filter_text: String::new(),
            show_closed: true,
            camera_scale: if ORTHOGRAPHIC_PROJECTION {
                // The default scale of an orthographic camera projection.
                1.0
            } else {
                // The default fov of a perspective camera projection.
                std::f32::consts::FRAC_PI_4
            },
            selected_node_id: None,
            input_debounce_timer: Timer::default(),
        }
    }
}

impl UiState {
    pub fn select_node(&mut self, node_id: NodeId) {
        self.selected_node_id = Some(node_id);
    }

    pub fn deselect_node(&mut self, _node_id: &NodeId) {
        self.selected_node_id = None;
    }
}

/// Send this event to request re-laying out everything in the scene.
#[derive(Debug, Default, Event)]
pub(crate) struct NeedsLayoutEvent {}

#[derive(Debug, Default, Event)]
pub(crate) struct FilterChangeEvent {}

#[derive(Debug, Default, Event)]
pub(crate) struct CameraChangeEvent {}

pub(crate) fn immediate_system(
    mut contexts: EguiContexts,
    mut state: ResMut<UiState>,
    mut needs_layout_events: EventWriter<NeedsLayoutEvent>,
    mut filter_events: EventWriter<FilterChangeEvent>,
    mut camera_events: EventWriter<CameraChangeEvent>,
    flowchart: Res<chart::Flowchart>,
    time: Res<Time>,
) {
    // Update the timer.
    state.input_debounce_timer.tick(time.delta());
    // Debounced input.
    if state.input_debounce_timer.just_finished() {
        needs_layout_events.send_default();
    }

    // When a filter changes, send a filter event and debounce the input for
    // animation purposes.
    fn debounce_filter_input(
        state: &mut UiState,
        filter_events: &mut EventWriter<FilterChangeEvent>,
    ) {
        filter_events.send_default();
        state
            .input_debounce_timer
            .set_duration(Duration::from_millis(1000));
        state.input_debounce_timer.reset();
    }

    egui::Window::new("View").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Filter");
            let filter_response =
                ui.text_edit_singleline(&mut state.filter_text);
            if filter_response.changed() {
                debounce_filter_input(&mut state, &mut filter_events);
            }
        });
        if ui.checkbox(&mut state.show_closed, "Show closed").changed() {
            debounce_filter_input(&mut state, &mut filter_events);
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Camera Scale (Zoom)");
            let mut z_position = egui::DragValue::new(&mut state.camera_scale);
            if ORTHOGRAPHIC_PROJECTION {
                z_position = z_position.clamp_range(0.3..=50.0).speed(0.1);
            } else {
                z_position = z_position
                    .clamp_range(0.15..=std::f32::consts::PI * 5.0 / 6.0)
                    .speed(0.01);
            }
            if ui.add(z_position).changed() {
                camera_events.send_default();
            }
            ui.label("Shift + Scroll Vertically");
        });
        ui.separator();
        if let Some(selected_node_id) = state.selected_node_id.as_ref() {
            if let Some(node) = flowchart.get_node_by_id(selected_node_id) {
                ui.label(node.text.as_str());
                ui.hyperlink(node.url.as_str());
            } else {
                ui.label("Node not found");
            }
        } else {
            ui.label("Nothing selected");
        }
    });
}

pub(crate) fn filter_events(
    state: Res<UiState>,
    mut filter_events: EventReader<FilterChangeEvent>,
    mut text_boxes_query: Query<(&TextBox, &mut Visibility)>,
) {
    let lower_case_filter = state.filter_text.to_lowercase();
    for _ in filter_events.read() {
        for (text_box, mut visible) in text_boxes_query.iter_mut() {
            *visible = if text_box.matches(&lower_case_filter)
                && (state.show_closed || text_box.is_state_open())
            {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub(crate) fn camera_events(
    state: Res<UiState>,
    mut camera_events: EventReader<CameraChangeEvent>,
    mut projection_query: Query<&mut Projection>,
) {
    for _ in camera_events.read() {
        for mut projection in projection_query.iter_mut() {
            match projection.as_mut() {
                Projection::Perspective(perspective) => {
                    perspective.fov = state.camera_scale;
                }
                Projection::Orthographic(orthographic) => {
                    orthographic.scale = state.camera_scale;
                }
            }
        }
    }
}
