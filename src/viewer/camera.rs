use crate::constants::EARTH_EQUATORIAL_RADIUS_KM;
use bevy::prelude::*;
use bevy_easings::*;
use smooth_bevy_cameras::{controllers::unreal::*, *};
use std::time::Duration;
use viewport_orientation_gizmo::*;

#[derive(Component)]
struct MainCamera;

pub struct CameraControlPlugin;
impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LookTransformPlugin)
            .add_plugin(UnrealCameraPlugin::default())
            .add_plugin(EasingsPlugin)
            .add_plugin(ViewportOrientationGizmoPlugin::custom(PluginOptions {
                gizmo: my_gizmo,
                ..default()
            }))
            .add_startup_system(setup)
            .add_system(react_to_clicks)
            .add_system_to_stage(CoreStage::PostUpdate, update_orth_zoom)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                enable_camera_controller_if_finished_animating,
            );
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            PerspectiveCameraBundle::default(),
            Vec3::new(0., 0., 3.5 * EARTH_EQUATORIAL_RADIUS_KM as f32 / 1000.),
            Vec3::new(0., 0., 0.),
        ))
        .insert(TrackedRotator);
}

fn update_orth_zoom(
    mut camera: Query<(&LookTransform, &mut OrthographicProjection), With<TrackedRotator>>,
) {
    for (look_tfm, mut orth_proj) in camera.iter_mut() {
        orth_proj.scale = (look_tfm.target - look_tfm.eye).length() / 1000.;
    }
}

#[allow(clippy::type_complexity)]
fn react_to_clicks(
    mut commands: Commands,
    mut events: EventReader<ClickEvent>,
    keyboard: Res<Input<KeyCode>>,
    mut camera: Query<
        (
            Entity,
            &Transform,
            &LookTransform,
            &mut UnrealCameraController,
            Option<&PerspectiveProjection>,
            Option<&OrthographicProjection>,
            Option<&EasingComponent<Transform>>,
        ),
        With<TrackedRotator>,
    >,
) {
    if camera.is_empty() {
        return;
    }

    const ANIM_DURATION: Duration = Duration::from_millis(1000);

    let (entity, tfm, look_tfm, mut controller, persp_proj, orth_proj, current_anim) =
        camera.single_mut();

    let dir = if keyboard.pressed(KeyCode::LControl) {
        -1.
    } else {
        1.
    };

    if let Some(current_anim) = current_anim {
        if current_anim.state == EasingState::Play {
            // Wait for animations to finish before allowing another input
            return;
        }
    }

    for event in events.iter() {
        if let Some(axis) = event.0 {
            let mut clicked_axis = dir * Vec3::from(axis);

            if clicked_axis.abs() == Vec3::Y {
                clicked_axis =
                    Quat::from_axis_angle(Vec3::X, clicked_axis.y.signum() * 0.01) * clicked_axis;
            }

            let distance_to_target = (look_tfm.target - tfm.translation).length();

            let mut new_tf: Transform = *tfm;

            // We already know where the eye will sit: on the clicked axis, at the same distance
            // we were before rotating.
            new_tf.translation = look_tfm.target + distance_to_target * clicked_axis;

            // Rotate!
            new_tf.look_at(look_tfm.target, Vec3::Y);

            // Align camera Z with selected direction
            commands.entity(entity).insert(tfm.ease_to(
                new_tf,
                EaseFunction::ExponentialOut,
                EasingType::Once {
                    duration: ANIM_DURATION,
                },
            ));

            controller.enabled = false;
        } else if let (Some(_), None) = (persp_proj, orth_proj) {
            commands
                .entity(entity)
                .remove::<PerspectiveProjection>()
                .insert(OrthographicProjection::default());
        } else if let (None, Some(_)) = (persp_proj, orth_proj) {
            commands
                .entity(entity)
                .remove::<OrthographicProjection>()
                .insert(PerspectiveProjection::default());
        }
    }
}

fn enable_camera_controller_if_finished_animating(
    mut controllers: Query<
        (&mut UnrealCameraController, &mut LookTransform, &Transform),
        Without<EasingComponent<Transform>>,
    >,
    removed: RemovedComponents<EasingComponent<Transform>>,
) {
    for e in removed.iter() {
        if let Ok((mut controller, mut look_tfm, tfm)) = controllers.get_mut(e) {
            look_tfm.eye = tfm.translation;
            controller.enabled = true;
        }
    }
}

gizmo![my_gizmo(meshes, materials):
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 1.0,
            max_y: 0.15,
            max_z: 0.15,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("b82700").unwrap(),
            unlit: true,
            ..default()
        }),
        transform: Transform::identity(),
        ..default()
    } ; RaycastableGizmo::default() ; GizmoClickableAxis::X,
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 0.15,
            max_y: 1.0,
            max_z: 0.15,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("5d9900").unwrap(),
            unlit: true,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    } ; RaycastableGizmo::default() ; GizmoClickableAxis::Y,
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 0.15,
            max_y: 0.15,
            max_z: 1.0,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("2e78e4").unwrap(),
            unlit: true,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    } ; RaycastableGizmo::default() ; GizmoClickableAxis::Z,
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: -0.15,
            min_y: -0.15,
            min_z: -0.15,
            max_x: 0.35,
            max_y: 0.35,
            max_z: 0.35,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::GRAY,
            unlit: true,
            ..default()
        }),
        transform: Transform::default(),
        ..default()
    } ; RaycastableGizmo::default(),
];
