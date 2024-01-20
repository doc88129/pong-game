use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

const PLAYER_COLOR: Color = Color::WHITE;
const PLAYER_SIZE: Option<Vec2> = Some(Vec2::new(20.0, 150.0));
const PLAYER_LOCATION: f32 = 450.;
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

// #[derive(Component)]
// struct ScoreBoard {
//     left_score: u32,
//     right_score: u32,
// }

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, scene_setup)
        .add_systems(Update, (player_input_system, ball_movement_system, collision_system))
        .run();
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
                ball_info.direction += 180.;
                ball_info.speed += 50.;
            }
            if ball_transform.translation.y.abs() > ceiling || ball_transform.translation.x.abs() > wall_right {
                ball_info.direction += 180.;
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

fn scene_setup(
    mut command: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    command.spawn(Camera2dBundle::default());

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

    // command.spawn((
    //     // Create a TextBundle that has a Text with a single section.
    //     TextBundle::from_section(
    //         "0 - 0",
    //         TextStyle {
    //             font_size: 60.0,
    //             ..default()
    //         },
    //     )
    //         .with_text_alignment(TextAlignment::Center)
    //         .with_style(Style {
    //             position_type: PositionType::Absolute,
    //             bottom: Val::Px(650.),
    //             right: Val::Px(550.),
    //             align_content: AlignContent::Center,
    //             ..default()
    //         }),
    //     ScoreBoard {
    //         left_score: 0,
    //         right_score: 0,
    //     },
    // ));
}