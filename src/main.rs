use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use rand::random_range;

#[derive(Component, Debug)]
struct Collider;

#[derive(Component, Debug)]
struct Paddle;

#[derive(Component, Debug)]
struct Brick;

#[derive(Component, Debug)]
struct Ball;

#[derive(Component, Debug)]
struct Velocity(Vec2);

const BALL_SPEED: f32 = 400.0;
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_SIZE: Vec2 = Vec2::new(150.0, 10.0);
const BRICK_SIZE: Vec2 = Vec2::new(135.0, 40.0);
const BALL_SIZE: f32 = 20.0;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.957, 0.953, 0.949)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_paddle,
                collide_ball,
                apply_velocity,
                collide_ball_with_walls,
            ),
        )
        .run();
}

const BRICK_GRID: UVec2 = UVec2::new(8, 6);
fn setup(mut commands: Commands, window: Query<&Window>) {
    let half_window_height = (window.single().resolution.height()) / 2.;
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
                        -((BRICK_GRID.x - 1) as f32 * (BRICK_SIZE.x + 5.) / 2.) + col as f32 * (BRICK_SIZE.x + 5.),
                        (half_window_height - (BRICK_SIZE.y * 2.)) - row as f32 * (BRICK_SIZE.y + 5.),
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
        Velocity(Vec2::splat(1.).normalize() * BALL_SPEED),
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
}

fn move_paddle(
    mut paddle: Single<&mut Transform, With<Paddle>>,
    window: Query<&Window>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let paddle_x = &mut paddle.translation.x;
    let half_window_width = (window.single().resolution.width() - PADDLE_SIZE.x) / 2.;
    if keys.pressed(KeyCode::KeyA) {
        *paddle_x -= PADDLE_SPEED * time.delta_secs();
    }
    if keys.pressed(KeyCode::KeyD) {
        *paddle_x += PADDLE_SPEED * time.delta_secs();
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

fn collide_ball(
    mut commands: Commands,
    mut ball: Query<(&Transform, &mut Velocity), With<Ball>>,
    colliders: Query<(&Transform, Entity, Option<&Brick>), With<Collider>>,
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

            if ball_bounds.intersects(&collider_bounds) {
                let collision_point = ball_bounds.closest_point(collider_bounds.center());
                
                if is_brick.is_some() {
                    commands.entity(entity).despawn();
                } 

                let offset = ball_bounds.center() - collision_point;
                if offset.x.abs() > offset.y.abs() && 
                   offset.x.is_sign_positive() != velocity.0.x.is_sign_positive() {
                    velocity.0.x *= -1.;
                } else if offset.y.is_sign_positive() != velocity.0.y.is_sign_positive() {
                    if is_brick.is_none() {
                        let pad_location = (ball_bounds.center().x - collider_bounds.center().x) / PADDLE_SIZE.x;
                        velocity.0 = Vec2::from_angle(pad_location * core::f32::consts::FRAC_PI_3 - core::f32::consts::FRAC_PI_2) * BALL_SPEED;
                        if keys.pressed(KeyCode::KeyA) {
                            velocity.0.x -= PADDLE_SPEED / 3.;
                        }
                        if keys.pressed(KeyCode::KeyD) {
                            velocity.0.x += PADDLE_SPEED / 3.;
                        }
                    }
                    velocity.0.y *= -1.;
                }
            }
        }
    }
}
