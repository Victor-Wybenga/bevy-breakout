use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use rand::random_range;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

#[derive(Component, Debug)]
struct Collider;

#[derive(Component, Debug)]
struct Paddle;

#[derive(Component, Debug)]
struct Brick;

#[derive(Component, Debug)]
struct Ball;

#[derive(Component, Debug)]
struct ScoreText;

#[derive(Component, Debug)]
struct StageText;

#[derive(Resource, Debug)]
struct State {
    score: u32,
    stage: u32,
}

#[derive(Component, Debug)]
struct Velocity(Vec2);

const BALL_SPEED: f32 = 400.0;
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_SIZE: Vec2 = Vec2::new(150.0, 10.0);
const BRICK_SIZE: Vec2 = Vec2::new(135.0, 40.0);
const BALL_SIZE: f32 = 20.0;
const STAGE_SPEED_FACTOR: f32 = 0.2;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                name: Some("Bevy Breakout".into()),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.957, 0.953, 0.949)))
        .insert_resource(State { score: 0, stage: 1 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_paddle,
                apply_velocity,
                collide_ball_with_walls,
                update_texts,
                ball_fall_through,
                reset_bricks,
                collide_ball
                .after(move_paddle),
            ),
        )
        .run();
}

const BRICK_GRID: UVec2 = UVec2::new(8, 6);
fn setup(mut commands: Commands, window: Query<&Window>) {
    let half_window_height = (window.single().resolution.height()) / 2.;
    let half_window_width = (window.single().resolution.width()) / 2.;
    commands.spawn(Camera2d);
    commands.spawn((
        Paddle,
        Collider,
        Transform {
            translation: Vec3::new(0., -300., 0.),
            scale: PADDLE_SIZE.extend(0.),
            ..default()
        },
        Sprite {
            color: Color::BLACK,
            ..default()
        },
    ));
    for row in 0..BRICK_GRID.y {
        for col in 0..BRICK_GRID.x {
            commands.spawn((
                Brick,
                Collider,
                Transform {
                    translation: Vec3::new(
                        -((BRICK_GRID.x - 1) as f32 * (BRICK_SIZE.x + 5.) / 2.)
                            + col as f32 * (BRICK_SIZE.x + 5.),
                        (half_window_height - (BRICK_SIZE.y * 2.))
                            - row as f32 * (BRICK_SIZE.y + 5.),
                        0.,
                    ),
                    scale: BRICK_SIZE.extend(0.),
                    ..default()
                },
                Sprite {
                    color: Color::hsl(random_range((0.)..360.), 0.6, 0.5),
                    ..default()
                },
            ));
        }
    }
    commands.spawn((
        Ball,
        Velocity(Vec2::splat(1.).normalize() * BALL_SPEED * (1. + STAGE_SPEED_FACTOR)),
        Transform {
            translation: Vec3::new(0., -250., 0.),
            scale: Vec2::splat(BALL_SIZE).extend(0.),
            ..default()
        },
        Sprite {
            color: Color::srgb(1., 0.412, 0.380),
            ..default()
        },
    ));
    commands
        .spawn((
            Text::new("Score: "),
            TextFont {
                font_size: 30.,
                ..default()
            },
            TextColor(Color::BLACK),
            Node {
                top: Val::Px(5.),
                left: Val::Px(5.),
                ..default()
            },
        ))
        .with_child((
            ScoreText,
            TextSpan::default(),
            TextFont {
                font_size: 30.,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.212, 0.180)),
        ));

    commands.spawn((
            Text::new("Stage: "),
            TextFont {
                font_size: 30.,
                ..default()
            },
            TextColor(Color::BLACK),
            Node {
                top: Val::Px(5.),
                left: Val::Px(half_window_width - 100.),
                ..default()
            },
        ))
        .with_child((
            StageText,
            TextSpan::default(),
            TextFont {
                font_size: 30.,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.212, 0.180)),
        ));
}

fn update_texts(
    state: Res<State>,
    mut text_set: ParamSet<(
        Single<&mut TextSpan, With<ScoreText>>,
        Single<&mut TextSpan, With<StageText>>,
    )>
) {
    text_set.p0().0 = format!("{}", state.score);
    text_set.p1().0 = format!("{}", state.stage);
}

fn move_paddle(
    mut paddle: Single<&mut Transform, With<Paddle>>,
    window: Query<&Window>,
    time: Res<Time>,
    state: Res<State>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let paddle_x = &mut paddle.translation.x;
    let half_window_width = (window.single().resolution.width() - PADDLE_SIZE.x) / 2.;
    if keys.pressed(KeyCode::KeyA) {
        *paddle_x -= PADDLE_SPEED * time.delta_secs() * (1. + state.stage as f32 * 0.15);
    }
    if keys.pressed(KeyCode::KeyD) {
        *paddle_x += PADDLE_SPEED * time.delta_secs() * (1. + state.stage as f32 * 0.15);
    }
    *paddle_x = paddle_x.clamp(-half_window_width, half_window_width);
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    window: Query<&Window>,
    time: Res<Time>,
) {
    let window = window.single();
    let window_size = Vec3::new(window.resolution.width(), window.resolution.height(), 0.);

    for (mut transform, Velocity(velocity)) in &mut query {
        transform.translation += velocity.extend(0.) * time.delta_secs();
        let half_window = (window_size - transform.scale) / 2.;
        transform.translation = transform
            .translation
            .clamp(-half_window.with_y(f32::INFINITY), half_window);
    }
}

fn reset_bricks(
    mut commands: Commands,
    brick_query: Query<&Brick>,
    mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    mut state: ResMut<State>,
    window: Query<&Window>,
) {
    let half_window_height = (window.single().resolution.height()) / 2.;
    if brick_query.iter().count() == 0 {
        state.stage += 1;
        ball.iter_mut().for_each(|(mut transform, mut velocity)| {
            transform.translation = Vec3::new(0., -250., 0.);
            velocity.0 = velocity.0.normalize()
                * BALL_SPEED
                * (1. + state.stage as f32 * STAGE_SPEED_FACTOR);
        });
        for row in 0..BRICK_GRID.y {
            for col in 0..BRICK_GRID.x {
                commands.spawn((
                    Brick,
                    Collider,
                    Transform {
                        translation: Vec3::new(
                            -((BRICK_GRID.x - 1) as f32 * (BRICK_SIZE.x + 5.) / 2.)
                                + col as f32 * (BRICK_SIZE.x + 5.),
                            (half_window_height - (BRICK_SIZE.y * 2.))
                                - row as f32 * (BRICK_SIZE.y + 5.),
                            0.,
                        ),
                        scale: BRICK_SIZE.extend(0.),
                        ..default()
                    },
                    Sprite {
                        color: Color::hsl(random_range((0.)..360.), 0.6, 0.5),
                        ..default()
                    },
                ));
            }
        }
    }
}

fn ball_fall_through(
    mut ball: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    window: Query<&Window>,
    mut state: ResMut<State>,
) {
    let half_window_height = (window.single().resolution.height()) / 2.;
    for (mut transform, mut velocity) in &mut ball {
        if transform.translation.y <= -half_window_height {
            transform.translation = Vec3::new(0., -250., 0.);
            velocity.0 *= -1.;
            state.score = state
                .score
                .checked_sub(state.stage * state.stage)
                .unwrap_or(0);
        }
    }
}

fn collide_ball_with_walls(
    mut ball: Query<(&Transform, &mut Velocity), With<Ball>>,
    window: Query<&Window>,
) {
    let window = window.single();
    let half_window = Vec3::new(
        window.resolution.width() - BALL_SIZE,
        window.resolution.height() - BALL_SIZE,
        0.,
    ) / 2.;

    for (transform, mut velocity) in ball.iter_mut() {
        if transform.translation.x.abs() >= half_window.x {
            velocity.0.x *= -1.;
        }
        if transform.translation.y >= half_window.y {
            velocity.0.y *= -1.;
        }
    }
}

#[derive(Debug)]
enum Collision {
    Horizontal,
    Vertical,
}

fn collides(object: &Aabb2d, with: &Aabb2d, velocity: Vec2) -> Option<Collision> {
    let swept_object = Aabb2d::from_point_cloud(Isometry2d::IDENTITY, &[object.min, object.max, (object.max + velocity), (object.min + velocity)]);
    if !swept_object.intersects(with) { return None }
    let collision_point = object.closest_point(with.center());
    let offset = object.center() - collision_point;

    if offset.x.abs() > offset.y.abs() {
        if velocity.x.is_sign_positive() == offset.x.is_sign_positive() {
            return None;
        };
        Some(Collision::Horizontal)
    } else {
        if velocity.y.is_sign_positive() == offset.y.is_sign_positive() {
            return None;
        };
        Some(Collision::Vertical)
    }
}

fn collide_ball(
    mut commands: Commands,
    mut ball: Query<(&Transform, &mut Velocity), With<Ball>>,
    colliders: Query<(&Transform, Entity, Option<&Brick>), With<Collider>>,
    mut state: ResMut<State>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (collider_transform, entity, is_brick) in &colliders {
        let collider_bounds = Aabb2d::new(
            collider_transform.translation.truncate(),
            collider_transform.scale.truncate() / 2.,
        );
        for (ball_transform, mut velocity) in &mut ball {
            let ball_bounds = Aabb2d::new(
                ball_transform.translation.truncate(),
                ball_transform.scale.truncate() / 2.,
            );

            let Some(collision) = collides(&ball_bounds, &collider_bounds, velocity.0 * time.delta_secs()) else {
                continue;
            };

            if is_brick.is_some() {
                commands.entity(entity).despawn();
                state.score += state.stage;
                match collision {
                    Collision::Horizontal => velocity.0.x *= -1.,
                    Collision::Vertical => velocity.0.y *= -1.,
                }
            } else {
                match collision {
                    Collision::Horizontal => velocity.0.x *= -1.,
                    Collision::Vertical => {
                        let pad_location =
                            (ball_bounds.center().x - collider_bounds.center().x) / PADDLE_SIZE.x;
                        velocity.0 = Vec2::from_angle(-(pad_location * FRAC_PI_3) + FRAC_PI_2)
                            * BALL_SPEED
                            * (1. + state.stage as f32 * STAGE_SPEED_FACTOR);

                        if keys.pressed(KeyCode::KeyA) {
                            velocity.0.x -= PADDLE_SPEED / 3.;
                        }
                        if keys.pressed(KeyCode::KeyD) {
                            velocity.0.x += PADDLE_SPEED / 3.;
                        }
                    }
                }
            }
        }
    }
}
