//! 2D Pixel-Perfect Camera and Touch/Desktop Navigation Module.
//!
//! Complies with `SPEC-REQ-FRONT-001` and `SPEC-REQ-FRONT-006`.

use bevy::app::{App, Plugin, Startup, Update};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::component::Component;
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::render::camera::OrthographicProjection;
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;

use crate::viewport::{compute_pixel_perfect_scale, ViewportGeometry, LOGICAL_WIDTH_MIN};

/// Component tagging the active 2D pixel camera.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PixelCamera;

/// Global camera configuration and user zoom state.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct CameraSettings {
    /// Discrete integer zoom multiplier (1x, 2x, 3x, 4x).
    pub zoom: u32,
    /// Keyboard panning velocity in virtual pixels per second.
    pub pan_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            zoom: 1,
            pan_speed: 250.0,
        }
    }
}

/// Touch tracking state for mobile pan and pinch-to-zoom gestures.
#[derive(Resource, Debug, Default, Clone)]
pub struct TouchTrackingState {
    /// Positions of active touch contacts indexed by touch identifier.
    touches: Vec<(u64, Vec2)>,
    /// Distance between primary two touches during previous frame.
    last_pinch_dist: Option<f32>,
}

/// Plugin initializing and driving the pixel-perfect 2D orthographic camera.
#[derive(Debug, Default, Clone, Copy)]
pub struct PixelCameraPlugin;

impl Plugin for PixelCameraPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<CameraSettings>() {
            app.init_resource::<CameraSettings>();
        }
        if !app.world().contains_resource::<TouchTrackingState>() {
            app.init_resource::<TouchTrackingState>();
        }
        app.add_systems(Startup, setup_pixel_camera_system);
        app.add_systems(
            Update,
            (
                update_camera_projection_system,
                camera_touch_navigation_system,
                camera_desktop_navigation_system,
            ),
        );
    }
}

/// Spawns the 2D orthographic camera with nearest-neighbor projection settings.
pub fn setup_pixel_camera_system(mut commands: Commands) {
    // Center of 24x24 tilemap: 12 * 16.0 = 192.0, -12 * 16.0 = -192.0
    commands.spawn((
        Camera2d,
        PixelCamera,
        Transform::from_xyz(192.0, -192.0, 999.0),
    ));
}

/// Computes integer scaling and configures the camera's orthographic scale.
///
/// Implements `SPEC-REQ-FRONT-001`:
/// $$\text{Scale} = \max\left(1, \min\left(\left\lfloor \frac{W_{\text{window}}}{320} \right\rfloor, \left\lfloor \frac{H_{\text{window}}}{240} \right\rfloor\right)\right)$$
pub fn update_camera_projection_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut OrthographicProjection, With<PixelCamera>>,
    settings: Res<CameraSettings>,
    mut geometry: Option<ResMut<ViewportGeometry>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let win_w = window.width() as u32;
    let win_h = window.height() as u32;
    if win_w == 0 || win_h == 0 {
        return;
    }

    let base_scale = compute_pixel_perfect_scale(win_w, win_h, LOGICAL_WIDTH_MIN);
    let total_scale = (base_scale * settings.zoom).max(1);

    if let Some(geom) = geometry.as_deref_mut() {
        *geom = ViewportGeometry::from_physical_dimensions(win_w, win_h, LOGICAL_WIDTH_MIN);
    }

    for mut projection in &mut camera_query {
        projection.scale = 1.0 / total_scale as f32;
    }
}

/// Processes touch events: single-finger drag panning and dual-finger pinch zoom.
///
/// Implements `SPEC-REQ-FRONT-006`.
pub fn camera_touch_navigation_system(
    mut touch_events: EventReader<TouchInput>,
    mut tracking: ResMut<TouchTrackingState>,
    mut settings: ResMut<CameraSettings>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<PixelCamera>>,
) {
    let Ok((mut transform, projection)) = camera_query.get_single_mut() else {
        return;
    };

    for ev in touch_events.read() {
        match ev.phase {
            TouchPhase::Started => {
                tracking.touches.retain(|(id, _)| *id != ev.id);
                tracking.touches.push((ev.id, ev.position));
            }
            TouchPhase::Moved => {
                if let Some(pos) = tracking.touches.iter_mut().find(|(id, _)| *id == ev.id) {
                    let delta = ev.position - pos.1;
                    pos.1 = ev.position;

                    // 1-finger panning
                    if tracking.touches.len() == 1 {
                        let world_dx = -delta.x * projection.scale;
                        let world_dy = delta.y * projection.scale;
                        transform.translation.x += world_dx;
                        transform.translation.y += world_dy;
                    }
                }

                // 2-finger pinch-to-zoom
                if tracking.touches.len() >= 2 {
                    let p0 = tracking.touches[0].1;
                    let p1 = tracking.touches[1].1;
                    let current_dist = (p0 - p1).length();

                    if let Some(last_dist) = tracking.last_pinch_dist {
                        let diff = current_dist - last_dist;
                        if diff > 40.0 && settings.zoom < 4 {
                            settings.zoom += 1;
                            tracking.last_pinch_dist = Some(current_dist);
                        } else if diff < -40.0 && settings.zoom > 1 {
                            settings.zoom -= 1;
                            tracking.last_pinch_dist = Some(current_dist);
                        }
                    } else {
                        tracking.last_pinch_dist = Some(current_dist);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                tracking.touches.retain(|(id, _)| *id != ev.id);
                if tracking.touches.len() < 2 {
                    tracking.last_pinch_dist = None;
                }
            }
        }
    }
}

/// Processes desktop inputs: WASD/arrow keys pan, mouse drag pan, and scroll zoom.
#[allow(clippy::too_many_arguments)]
pub fn camera_desktop_navigation_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut contexts: EguiContexts,
    mut settings: ResMut<CameraSettings>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<PixelCamera>>,
) {
    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
        return;
    }

    let Ok((mut transform, projection)) = camera_query.get_single_mut() else {
        return;
    };

    // Keyboard Pan
    let mut move_dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }

    if move_dir != Vec2::ZERO {
        let delta = move_dir.normalize() * settings.pan_speed * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }

    // Mouse Drag Pan (Left or Middle click held)
    if mouse_button.pressed(MouseButton::Middle) || mouse_button.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            let world_dx = -ev.delta.x * projection.scale;
            let world_dy = ev.delta.y * projection.scale;
            transform.translation.x += world_dx;
            transform.translation.y += world_dy;
        }
    } else {
        mouse_motion.clear();
    }

    // Mouse Wheel Zoom
    for ev in mouse_wheel.read() {
        if ev.y > 0.0 && settings.zoom < 4 {
            settings.zoom += 1;
        } else if ev.y < 0.0 && settings.zoom > 1 {
            settings.zoom -= 1;
        }
    }
}
