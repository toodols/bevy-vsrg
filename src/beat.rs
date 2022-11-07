use crate::TrackInfo;
use bevy::prelude::*;
use chrono::Duration;
use std::ops::Deref;

#[derive(Component)]
pub struct Beat {
    pub track: usize,
    pub hit_at: Duration,
}

impl Beat {
    pub fn distance_from_line(&self, info: impl AsRef<TrackInfo>) -> Duration {
        (info.as_ref().start_at + self.hit_at) - chrono::Local::now()
    }
}

// fn t(){
//     fn a<'a>(a: impl AsRef<TrackInfo>){
//         let s = a.as_ref().start_at;
//     }
//     fn any<T>() -> T {
//         unsafe {
//             *(0 as *const T)
//         }
//     }
//     let b = any::<Res<TrackInfo>>();
//     let c = Deref::deref(&b);
//     a(&b);
// }

pub fn beat_system(
    mut commands: Commands,
    mut query: Query<(&Beat, &mut Transform, Entity)>,
    info: Res<TrackInfo>,
) {
    for (beat, mut transform, entity) in &mut query {
        let d = beat.distance_from_line(&info);
        transform.translation.y = d.num_milliseconds() as f32 / 3.0 + 20.0;
        if d < Duration::milliseconds(-1000) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn create_beat() {}
