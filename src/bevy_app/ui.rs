use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::chart;
use crate::chart::NodeId;

use super::text_box::TextBox;

#[derive(Debug, Default, Resource)]
pub(crate) struct UiState {
    filter_text: String,
    selected_node_id: Option<NodeId>,
}

impl UiState {
    pub fn select_node(&mut self, node_id: NodeId) {
        self.selected_node_id = Some(node_id);
    }

    pub fn deselect_node(&mut self, _node_id: &NodeId) {
        self.selected_node_id = None;
    }
    // pub fn toggle_selected_node(&mut self, node_id: NodeId) {
    //     if self.selected_node_id.as_ref() == Some(&node_id) {
    //         self.selected_node_id = None;
    //     } else {
    //         self.selected_node_id = Some(node_id);
    //     }
    // }
}

#[derive(Debug, Default, Event)]
pub(crate) struct FilterChangeEvent {}

pub(crate) fn immediate_system(
    mut contexts: EguiContexts,
    mut state: ResMut<UiState>,
    mut filter_events: EventWriter<FilterChangeEvent>,
    flowchart: Res<chart::Flowchart>,
) {
    egui::Window::new("View").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Filter");
            let filter_response =
                ui.text_edit_singleline(&mut state.filter_text);
            if filter_response.changed() {
                filter_events.send_default();
            }
        });
        ui.separator();
        if let Some(selected_node_id) = state.selected_node_id.as_ref() {
            if let Some(node) = flowchart.get_node_by_id(selected_node_id) {
                ui.label(node.text.as_str());
                ui.label(node.url.as_str());
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
            {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
