mod beat;
mod layer;
mod line;
mod resized;

use std::fmt;

use beat::{beat_system, Beat};
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    time::FixedTimestep,
};
use bevy_prototype_lyon::prelude::*;
use chrono::{DateTime, Duration};
use layer::Layer;
use line::create_line;
use rand::Rng;
use resized::resized_system;

pub struct TrackInfo {
    start_at: DateTime<chrono::Local>,
    track_count: usize, //this won't be needed
}

#[derive(Component)]
struct ScoreDisplay;

struct Score(i32);

struct DefaultFont(Handle<Font>);

struct HitSound(Handle<AudioSource>);
struct HitSoundEvent;

#[derive(Component)]
pub struct BottomEdge;

#[derive(Component)]
enum HitResponse {
    Perfect,
    Good,
    Okay,
    Meh,
    Miss,
}

impl fmt::Display for HitResponse {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HitResponse::Perfect => write!(fmt, "Perfect!"),
            HitResponse::Good => write!(fmt, "Good!"),
            HitResponse::Okay => write!(fmt, "Okay!"),
            HitResponse::Meh => write!(fmt, "Meh!"),
            HitResponse::Miss => write!(fmt, "Miss!"),
        }
    }
}

impl HitResponse {
    fn color(&self) -> Color {
        match self {
            HitResponse::Perfect => Color::rgb(0.0, 1.0, 1.0),
            HitResponse::Good => Color::rgb(0.0, 0.8, 0.8),
            HitResponse::Okay => Color::rgb(0.0, 1.0, 0.0),
            HitResponse::Meh => Color::rgb(0.8, 0.7, 0.0),
            HitResponse::Miss => Color::rgb(1.0, 0.0, 0.0),
        }
    }
}

impl From<Duration> for HitResponse {
    fn from(d: Duration) -> Self {
        // basically absolute value
        let d = if d < Duration::zero() {
            -d
        } else {
            d
        };

        if d < Duration::milliseconds(15) {
            HitResponse::Perfect
        } else if d < Duration::milliseconds(30) {
            HitResponse::Good
        } else if d < Duration::milliseconds(50) {
            HitResponse::Okay
        } else if d < Duration::milliseconds(100) {
            HitResponse::Meh
        } else {
            HitResponse::Miss
        }
    }
}

fn hit_response_sys(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Text, Entity), With<HitResponse>>,
) {
    for (mut transform, mut text, entity) in &mut query {
        let color: &mut Color = &mut text.sections[0].style.color;
        let new_alpha = color.a() - 0.1;
        if new_alpha < 0.0 {
            commands.entity(entity).despawn();
        } else {
            color.set_a(new_alpha);
            transform.translation.y += 1.0;
        }
    }
}

#[derive(Bundle)]
struct HitResponseBundle {
    #[bundle]
    text_bundle: Text2dBundle,
    response: HitResponse,
}

fn hit_response(font: Handle<Font>, res: HitResponse) -> HitResponseBundle {
    HitResponseBundle {
        text_bundle: Text2dBundle {
            text: Text::from_section(
                res.to_string(),
                TextStyle {
                    font,
                    font_size: 15.0,
                    color: res.color(),
                },
            )
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
            ..default()
        },
        response: res,
    }
}

fn setup_system(mut commands: Commands, mut windows: ResMut<Windows>, assets: Res<AssetServer>) {
    commands.insert_resource(HitSound(assets.load("hitsound.wav")));
    commands.insert_resource(DefaultFont(assets.load("Arial.ttf")));
    if let Some(win) = windows.get_primary_mut() {
        win.set_title("Rhythm Game".to_string());
    }
    commands.spawn().insert(ScoreDisplay).insert_bundle(Text2dBundle{
        text: Text::from_section(
            "Score: 0",
            TextStyle {
                font: assets.load("Arial.ttf"),
                font_size: 15.0,
                color: Color::rgb(0.0, 0.0, 0.0),
            },
        ),
        transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
        ..default()
    });
    commands
        .spawn()
        .insert(BottomEdge)
        .insert_bundle(SpatialBundle::default())
        .with_children(|parent| {
            let mut rng = rand::thread_rng();
            for i in 10..200 {
                let n = rng.gen_range(0..4) as usize;
                let shape = shapes::Circle {
                    radius: 10.0,
                    center: Vec2::ZERO,
                };
                parent
                    .spawn()
                    // .insert_bundle(SpatialBundle::default())
                    .insert(Beat {
                        track: n,
                        hit_at: Duration::milliseconds(i * 250),
                    })
                    .insert_bundle(GeometryBuilder::build_as(
                        &shape,
                        DrawMode::Fill(FillMode::color(match n {
                            0 => Color::rgb(1., 0., 0.),
                            1 => Color::rgb(0., 1., 0.),
                            2 => Color::rgb(0., 0., 1.),
                            3 => Color::rgb(1., 1., 0.),
                            _ => Color::rgb(0., 0., 0.),
                        })),
                        Transform::from_translation(Vec3::new(
                            (n as f32 - 2.0) * 50.0 + 25.0,
                            10.0,
                            Layer::BEATS,
                        )),
                    ));
            }
            parent.spawn_bundle(create_line());
        });
    commands.spawn_bundle(Camera2dBundle::default());
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(&Beat, &mut Transform, &Parent, Entity)>,
    bottom_edge: Query<Entity, With<BottomEdge>>,
    mut hit_event: EventWriter<HitSoundEvent>,
    info: Res<TrackInfo>,
    font: Res<DefaultFont>,
    mut score: ResMut<Score>,
) {
    let bottom_edge: Entity = bottom_edge.iter().next().unwrap();
    for num in 0..4 {
        if keyboard_input.just_pressed(match num {
            0 => KeyCode::D,
            1 => KeyCode::F,
            2 => KeyCode::J,
            3 => KeyCode::K,
            _ => panic!("Invalid key"),
        }) {
            if let Some((beat, mut transform, parent, entity)) = (&mut query)
                .into_iter()
                .filter(|(beat, transform, ..)| {
                    /*println!("{:?}", transform);*/
                    beat.track as i32 == num
                        && beat.distance_from_line(&info) < Duration::milliseconds(100)
                })
                .next()
            {
                let res = HitResponse::from(beat.distance_from_line(&info));
                score.0+=match res {
                    HitResponse::Perfect => 100,
                    HitResponse::Good => 50,
                    HitResponse::Okay => 25,
                    HitResponse::Meh => 10,
                    HitResponse::Miss => 0,
                };
                let id = commands
                    .spawn_bundle(hit_response(
                        font.0.clone(),
                        res,
                    ))
                    .id();
                commands.entity(bottom_edge).add_child(id);
                hit_event.send(HitSoundEvent);
                commands.entity(entity).despawn();
            }
        }
    }
}

fn play_hitsound(events: EventReader<HitSoundEvent>, audio: Res<Audio>, sound: Res<HitSound>) {
    if !events.is_empty() {
        events.clear();
        audio.play(sound.0.clone());
    }
}

fn score_display(mut query: Query<&mut Text, With<ScoreDisplay>>, score: Res<Score>) {
    for mut text in &mut query {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(TrackInfo {
            start_at: chrono::Local::now(),
            track_count: 4,
        })
        .insert_resource(Score(0))
        .add_event::<HitSoundEvent>()
        .add_startup_system(setup_system)
        .add_system(resized_system)
        .add_system(input_system)
        .add_system(score_display)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(beat_system),
        )
        .add_system(play_hitsound)
        .add_system(hit_response_sys)
        .run();
}
