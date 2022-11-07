use bevy::{prelude::*, window::WindowResized};

use crate::BottomEdge;

pub fn resized_system(
    mut event: EventReader<WindowResized>,
    mut bottom_edge: Query<(&BottomEdge, &mut Transform)>,
) {
    if let Some(event) = event.iter().last() {
        bottom_edge.single_mut().1.translation.y = -event.height / 2.0;
    }
}
