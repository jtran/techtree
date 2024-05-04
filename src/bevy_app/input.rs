use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// Keyboard controls the camera.
pub(crate) fn keyboard_system(
    mut camera_query: Query<
        &mut Transform,
        (With<Camera>, With<super::SceneCamera>),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut transform in camera_query.iter_mut() {
            transform.translation += Vec3::new(0.0, 0.1, 0.0);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        for mut transform in camera_query.iter_mut() {
            transform.translation += Vec3::new(0.0, -0.1, 0.0);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        for mut transform in camera_query.iter_mut() {
            transform.translation += Vec3::new(-0.1, 0.0, 0.0);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        for mut transform in camera_query.iter_mut() {
            transform.translation += Vec3::new(0.1, 0.0, 0.0);
        }
    }
}

/// Mouse wheel zooms.
pub(crate) fn events_system(
    mut projections: Query<&mut Projection>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    mut wheel: EventReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<super::ui::UiState>,
) {
    for event in wheel.read() {
        if keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            // Zoom.
            let mut projection = projections.single_mut();
            match projection.as_mut() {
                Projection::Perspective(perspective) => {
                    perspective.fov += match event.unit {
                        bevy::input::mouse::MouseScrollUnit::Line => {
                            event.y * 5_f32
                        }
                        bevy::input::mouse::MouseScrollUnit::Pixel => {
                            event.y * 0.01_f32
                        }
                    };
                    // Clamp.
                    perspective.fov = perspective
                        .fov
                        .max(0.15_f32)
                        .min(std::f32::consts::PI * 5.0 / 6.0);
                    // Copy to UI state.
                    state.camera_scale = perspective.fov;
                }
                Projection::Orthographic(orthographic) => {
                    orthographic.scale += match event.unit {
                        bevy::input::mouse::MouseScrollUnit::Line => {
                            event.y * 0.5_f32
                        }
                        bevy::input::mouse::MouseScrollUnit::Pixel => {
                            event.y * 0.01_f32
                        }
                    };
                    // Clamp.
                    orthographic.scale =
                        orthographic.scale.max(0.3_f32).min(50_f32);
                    // Copy to UI state.
                    state.camera_scale = orthographic.scale;
                }
            }
        } else {
            // Pan.
            for mut transform in camera_transform.iter_mut() {
                transform.translation +=
                    Vec3::new(-event.x * 0.02, event.y * 0.02, 0.0);
            }
        }
    }
}
