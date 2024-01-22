#![windows_subsystem = "windows"]

use std::time::Duration;
use bevy::{
    prelude::*,
    time::Stopwatch,
    sprite::{
        MaterialMesh2dBundle,
        collide_aabb::{collide, Collision}
    }
};
use rand::seq::SliceRandom;

const PLAYER_COLOR: Color = Color::WHITE;
const PLAYER_SIZE: Option<Vec2> = Some(Vec2::new(20.0, 150.0));
const PLAYER_LOCATION: f32 = 450.;

const BALL_STARTING_SPEED: f32 = 175.;
const PLAYER_SPEED: f32 = 5.;
const SPEED_INCREASE_PER_BOUNCE: f32 = 1.1;

const MAX_PLAYER_HEIGHT: f32 = 275.;
const GOAL_BUFFER: f32 = 75.;
const BALL_RADIUS: f32 = 10.;

#[derive(Event, Default)]
struct CollisionEvent(bool);

#[derive(Component)]
struct PlayerControlled;

#[derive(Component, Default, Deref, DerefMut)]
struct GameBall(Vec2);

#[derive(Component)]
struct TimerUI(Stopwatch);

#[derive(Resource, Default)]
struct ScoreBoard {
    left_score: u32,
    right_score: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ScoreBoard>()
        .add_event::<CollisionEvent>()
        .add_systems(Startup, scene_setup)
        .add_systems(FixedUpdate, (player_input_system, ball_movement_system, collision_system))
        .add_systems(Update, (score_event_trigger, collision_event_listener, timer_ui_system))
        .add_systems(Update, (reset_scene, update_scoreboard_system).run_if(resource_changed::<ScoreBoard>()))
        .run();
}

fn scene_setup(
    mut command: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    command.spawn(Camera2dBundle::default());

    paddle_setup(&mut command);
    ui_text_setup(&mut command);
    game_ball_setup(&mut command, meshes, materials);
}

fn ui_text_setup(command: &mut Commands) {
    command.spawn((
        TextBundle::from_section(
            "0 - 0",
            TextStyle {
                font_size: 60.0,
                ..default()
            },
        )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(650.),
                right: Val::Px(550.),
                align_content: AlignContent::Center,
                ..default()
            }),
    ));

    command.spawn((
        TextBundle::from_section(
            "Time: xx.xxx",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                position_type: PositionType::Relative,
                bottom: Val::Px(-690.),
                right: Val::Px(-10.),
                align_content: AlignContent::Start,
                ..default()
            }),
        TimerUI(Stopwatch::default()),
    ));
}

fn game_ball_setup(
    command: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    command.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    },
        GameBall(Vec2::new(
            *[BALL_STARTING_SPEED/2., -BALL_STARTING_SPEED/2.].choose(&mut rand::thread_rng()).unwrap(),
            *[BALL_STARTING_SPEED, -BALL_STARTING_SPEED].choose(&mut rand::thread_rng()).unwrap(),
        ))
    ));
}

fn paddle_setup(command: &mut Commands) {
    command.spawn((SpriteBundle {
        sprite: Sprite {
            color: PLAYER_COLOR,
            custom_size: PLAYER_SIZE,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(PLAYER_LOCATION, 0., 0.)),
        ..default()
    },
        PlayerControlled
    ));
    command.spawn((SpriteBundle {
        sprite: Sprite {
            color: PLAYER_COLOR,
            custom_size: PLAYER_SIZE,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-PLAYER_LOCATION, 0., 0.)),
        ..default()
    },
        PlayerControlled
    ));
}

fn reset_scene(
    mut paddle_query: Query<&mut Transform, Without<GameBall>>,
    mut ball_query: Query<(&mut Transform, &mut GameBall), With<GameBall>>,
    mut timer: Query<&mut TimerUI>,
) {
    for mut paddle_transform in &mut paddle_query {
        paddle_transform.translation.y = 0.;
    }

    let (mut ball_transform, mut ball_info) = ball_query.single_mut();
    ball_transform.translation.x = 0.;
    ball_transform.translation.y = 0.;
    ball_info.y = *[BALL_STARTING_SPEED/2., -BALL_STARTING_SPEED/2.].choose(&mut rand::thread_rng()).unwrap();
    ball_info.x = *[BALL_STARTING_SPEED, -BALL_STARTING_SPEED].choose(&mut rand::thread_rng()).unwrap();

    timer.single_mut().0.reset();
}

fn score_event_trigger(
    mut ball: Query<&Transform, With<GameBall>>,
    mut score_board: ResMut<ScoreBoard>,
    mut events: EventWriter<CollisionEvent>,
) {
    let ball_transform = ball.single_mut();

    if ball_transform.translation.x > PLAYER_LOCATION + GOAL_BUFFER {
        score_board.left_score += 1;
        events.send(CollisionEvent(true));
    } else if ball_transform.translation.x < -(PLAYER_LOCATION + GOAL_BUFFER) {
        score_board.right_score += 1;
        events.send(CollisionEvent(true));
    }
}

fn update_scoreboard_system(
    mut query: Query<&mut Text, Without<TimerUI>>,
    score_board: Res<ScoreBoard>,
) {
    let updated_text = format!("{} - {}", score_board.left_score ,score_board.right_score);
    for mut ui in &mut query {
        ui.sections[0].value = updated_text.clone();
    }
}

fn collision_system(
    windows: Query<&Window>,
    paddle_query: Query<&Transform, With<PlayerControlled>>,
    mut ball_query: Query<(&Transform, &mut GameBall), Without<PlayerControlled>>,
    mut events: EventWriter<CollisionEvent>
) {
    let window = windows.single();

    let ceiling = window.height() / 2.;
    let wall_right = window.width() / 2.;

    let (ball_transform, mut ball_info) = ball_query.single_mut();

    for paddle_transform in &paddle_query {
        let mut collision = collide(
            ball_transform.translation,
            Vec2::new(BALL_RADIUS, BALL_RADIUS),
            paddle_transform.translation,
            PLAYER_SIZE.unwrap(),
        );
        if ball_transform.translation.y > ceiling {
            collision = Some(Collision::Bottom);
        } else if ball_transform.translation.y < -ceiling {
            collision = Some(Collision::Top);
        } else if ball_transform.translation.x > wall_right {
            collision = Some(Collision::Left);
        }  else if ball_transform.translation.x < -wall_right {
            collision = Some(Collision::Right);
        }

        if let Some(collision) = collision {
            let mut vert_reflect = false;
            let mut hori_reflect = false;

            match collision {
                Collision::Left => hori_reflect = ball_info.x > 0.0,
                Collision::Right => hori_reflect = ball_info.x < 0.0,
                Collision::Top => vert_reflect = ball_info.y < 0.0,
                Collision::Bottom => vert_reflect = ball_info.y > 0.0,
                Collision::Inside => {}
            }

            if hori_reflect {
                ball_info.x = -ball_info.x * SPEED_INCREASE_PER_BOUNCE;
            }
            if vert_reflect {
                ball_info.y = -ball_info.y * SPEED_INCREASE_PER_BOUNCE;
            }

            events.send(CollisionEvent(false));
        }
    }
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerControlled>>,
) {
    for mut player in &mut query {
        if keyboard_input.pressed(KeyCode::Up) && player.translation.y < MAX_PLAYER_HEIGHT {
            player.translation.y += PLAYER_SPEED;
        } else if keyboard_input.pressed(KeyCode::Down) && player.translation.y > -MAX_PLAYER_HEIGHT{
            player.translation.y -= PLAYER_SPEED;
        }
    }
}

fn ball_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &GameBall)>,
) {
    let (mut ball_transform, ball_info) = query.single_mut();
    ball_transform.translation.x += ball_info.x * time.delta_seconds();
    ball_transform.translation.y += ball_info.y * time.delta_seconds();
}

fn collision_event_listener(
    mut commamds: Commands,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut events: EventReader<CollisionEvent>,
) {
    for collision in events.read() {
        if collision.0 {
            commamds.spawn(PitchBundle {
                source: pitch_assets.add(Pitch::new(100., Duration::new(0, 200_000_000))),
                settings: PlaybackSettings::DESPAWN,
            });
            commamds.spawn(PitchBundle {
                source: pitch_assets.add(Pitch::new(500., Duration::new(0, 200_000_000))),
                settings: PlaybackSettings::DESPAWN,
            });
        } else {
            commamds.spawn(PitchBundle {
                source: pitch_assets.add(Pitch::new(520., Duration::new(0, 20_000_000))),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

fn timer_ui_system(mut query: Query<(&mut Text, &mut TimerUI)>, time: Res<Time>) {
    let (mut ui, mut component) = query.single_mut();
    component.0.tick(time.delta());
    ui.sections[0].value = format!("Time: {}.{}",
                                   component.0.elapsed().as_secs(),
                                   component.0.elapsed().as_millis() % 100
    );
}
