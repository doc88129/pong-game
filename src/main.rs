use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use std::f32::consts::PI;

const PLAYER_COLOR: Color = Color::WHITE;
const PLAYER_SIZE: Option<Vec2> = Some(Vec2::new(20.0, 150.0));
const PLAYER_LOCATION: f32 = 450.;
const GOAL_BUFFER: f32 = 50.;
const PLAYER_SPEED: f32 = 2.;
const MAX_PLAYER_HEIGHT: f32 = 250.;
const BALL_STARTING_SPEED: f32 = 150.;
const BALL_RADIUS: f32 = 10.;

#[derive(Component)]
struct Collision;

#[derive(Component)]
struct PlayerControlled;

#[derive(Component)]
struct GameBall {
    speed: f32,
    direction: f32,
}

enum WinnerSide {
    Right,
    Left,
}
#[derive(Event)]
struct ScoreEvent {
    winner: WinnerSide
}

#[derive(Resource, Default)]
struct ScoreBoard {
    left_score: f32,
    right_score: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ScoreBoard>()
        .add_event::<ScoreEvent>()
        .add_systems(Startup, scene_setup)
        .add_systems(Update, (player_input_system, ball_movement_system, collision_system, score_event_trigger, score_event_listener))
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
        // Create a TextBundle that has a Text with a single section.
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
                   GameBall {
                       speed: BALL_STARTING_SPEED,
                       direction: 0.,
                   },
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
                   PlayerControlled,
                   Collision
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
                   PlayerControlled,
                   Collision
    ));
}

fn reset_scene(
    mut paddle_query: Query<&mut Transform, Without<GameBall>>,
    mut ball_query: Query<(&mut Transform, &mut GameBall), With<GameBall>>,
) {
    for mut paddle_transform in &mut paddle_query {
        paddle_transform.translation.y = 0.;
    }
    for (mut ball_transform, mut ball_info) in &mut ball_query {
        ball_transform.translation.x = 0.;
        ball_transform.translation.y = 0.;
        ball_info.direction = 0.;
        ball_info.speed = BALL_STARTING_SPEED;
    }
}

fn score_event_trigger(
    mut events: EventWriter<ScoreEvent>,
    ball: Query<&Transform, With<GameBall>>
) {
    let ball_transform = ball.single();
    if ball_transform.translation.x > PLAYER_LOCATION + GOAL_BUFFER {
        events.send(ScoreEvent {
            winner: WinnerSide::Left
        });
    } else if ball_transform.translation.x < -(PLAYER_LOCATION + GOAL_BUFFER) {
        events.send(ScoreEvent {
            winner: WinnerSide::Right
        });
    }
}

fn score_event_listener(
    mut events: EventReader<ScoreEvent>,
    mut score_board: ResMut<ScoreBoard>
) {
    for event in events.read() {
        match event.winner {
            WinnerSide::Right => {
                score_board.right_score += 0.5;
            },
            WinnerSide::Left => {
                score_board.left_score += 0.5;
            }
        }
    }
}


fn update_scoreboard_system(
    mut query: Query<&mut Text>,
    score_board: Res<ScoreBoard>,
) {
    let updated_text = format!("{} - {}", score_board.left_score ,score_board.right_score);
    for mut ui in &mut query {
        ui.sections[0].value = updated_text.clone();
    }
}

fn collision_system(
    windows: Query<&Window>,
    paddle_query: Query<&Transform, With<Collision>>,
    mut ball_query: Query<(&Transform, &mut GameBall), Without<Collision>>
) {
    let window = windows.single();

    let ceiling = window.height() / 2.;
    let wall_right = window.width() / 2.;
    for paddle_transform in &paddle_query {
        for (ball_transform, mut ball_info) in &mut ball_query {
            if (paddle_transform.translation.x - ball_transform.translation.x).abs() < BALL_RADIUS + 10.
                && (paddle_transform.translation.y - ball_transform.translation.y).abs() < BALL_RADIUS + 75.
            {
                ball_info.direction += PI / 4.;
                ball_info.speed += 50.;
            }
            if ball_transform.translation.y.abs() > ceiling || ball_transform.translation.x.abs() > wall_right {
                ball_info.direction += PI / 4.;
            }
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
    mut query: Query<(&mut Transform,&GameBall)>,
) {
    for (mut ball_transform, ball_info) in &mut query {
        ball_transform.translation.x += ball_info.speed * f32::cos(ball_info.direction) * time.delta_seconds();
        ball_transform.translation.y += ball_info.speed * f32::sin(ball_info.direction) * time.delta_seconds();
    }
}
