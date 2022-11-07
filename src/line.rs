use bevy::prelude::*;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};

use crate::layer::Layer;

#[derive(Component)]
struct Line;

#[derive(Bundle)]
pub struct LineBundle {
    line: Line,
    #[bundle]
    shape: ShapeBundle,
}
pub fn create_line() -> LineBundle {
    LineBundle {
        line: Line,
        shape: GeometryBuilder::build_as(
            &shapes::Line(Vec2::new(-100.0, 0.0), Vec2::new(100.0, 0.0)),
            DrawMode::Stroke(StrokeMode::color(Color::rgb(0., 0., 0.))),
            Transform::from_translation(Vec3::new(0.0, 20.0, Layer::LINE)),
        ),
    }
}
