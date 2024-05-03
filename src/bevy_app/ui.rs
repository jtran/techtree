use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::chart;
use crate::chart::NodeId;

use super::text_box::TextBox;

#[derive(Debug, Resource)]
pub(crate) struct UiState {
    filter_text: String,
    show_closed: bool,
    selected_node_id: Option<NodeId>,
    input_debounce_timer: Timer,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            filter_text: String::new(),
            show_closed: true,
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

pub(crate) fn immediate_system(
    mut contexts: EguiContexts,
    mut state: ResMut<UiState>,
    mut needs_layout_events: EventWriter<NeedsLayoutEvent>,
    mut filter_events: EventWriter<FilterChangeEvent>,
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
