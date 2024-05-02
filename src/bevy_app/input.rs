use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// pub(crate) fn mouse_button_system(mouse_button_input: Res<ButtonInput<MouseButton>>) {
//     if mouse_button_input.just_pressed(MouseButton::Left) {
//         println!("Left mouse button pressed");
//     }
// }

/// Keyboard controls the camera.
pub(crate) fn keyboard_system(
    mut camera_query: Query<&mut Transform, (With<Camera>, With<super::SceneCamera>)>,
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
    mut wheel: EventReader<MouseWheel>,
) {
    for event in wheel.read() {
        let mut projection = projections.single_mut();
        match projection.as_mut() {
            Projection::Perspective(perspective) => {
                perspective.fov += match event.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => event.y * 5_f32,
                    bevy::input::mouse::MouseScrollUnit::Pixel => event.y * 0.01_f32,
                };
                // Clamp.
                perspective.fov = perspective
                    .fov
                    .max(0.15_f32)
                    .min(std::f32::consts::PI * 5.0 / 6.0);
            }
            Projection::Orthographic(orthographic) => {
                orthographic.scale += match event.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => event.y * 0.5_f32,
                    bevy::input::mouse::MouseScrollUnit::Pixel => event.y * 0.01_f32,
                };
                // Clamp.
                orthographic.scale = orthographic.scale.max(0.5_f32).min(8_f32);
            }
        }
    }
}
