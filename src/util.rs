use bevy::prelude::*;

use crate::{Pipe, PipeBundle, PipePosition};
const PIPE_OFFSET: f32 = 30.0;
const PIPE_WIDTH: f32 = 100.0;

/**
 * Returns a rectangle
 */

pub fn get_pipe_from_position_and_size(
    size: f32,
    pos: PipePosition,
    camera_translation: Vec3,
    window_height: f32,
    window_width: f32,
) -> PipeBundle {
    let pipe_y_pos = match pos {
        PipePosition::Top => (window_height / 2.0) - (size / 2.0),
        PipePosition::Bottom => -(window_height / 2.0) + (size / 2.0),
    };

    PipeBundle {
        _p: Pipe::new(pos),
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(PIPE_WIDTH, size)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                camera_translation.x + (window_width / 2.0) + PIPE_OFFSET,
                pipe_y_pos,
                camera_translation.z,
            )),
            ..default()
        },
        ..default()
    }
}
