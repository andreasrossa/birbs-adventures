use bevy::prelude::*;

use crate::physics::Velocity;

#[derive(Component)]
pub struct Birb;

const MIN: f32 = -800.0;
const MAX: f32 = 800.0;
const ROTATION_RANGE: f32 = 160.0; // in degrees
pub fn rotate_birb(mut query: Query<(&mut Transform, &Velocity), With<Birb>>) {
    for (mut transform, velocity) in query.iter_mut() {
        // Clamp y velocity from -800 to 800 (1600 range)
        let clamped_velocity = velocity.0.y.clamp(MIN, MAX);

        // Calculate the rotation percentage (percentage of how much the birb should rotate)
        let rotation_percentage = (clamped_velocity + MIN.abs()) / (MAX - MIN);

        let z_rotation = -90.0 + rotation_percentage * ROTATION_RANGE;

        // println!(
        //     "{} {} {}",
        //     clamped_velocity, rotation_percentage, z_rotation
        // );
        transform.rotation = Quat::from_rotation_z(z_rotation.to_radians());
    }
}
