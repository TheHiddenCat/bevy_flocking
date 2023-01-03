use bevy::{prelude::*, time::FixedTimestep, render::camera::RenderTarget};

use rand::prelude::*;

const PHYSICS_STEP: f32 = 1.0 / 60.0;
const ANIMATE_STEP: f32 = 1.0 / 8.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                window: WindowDescriptor {
                    width: 1600.0,
                    height: 900.0,
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    title: "Birbs".to_owned(),
                    ..default()
                },
                ..default()
            })
        )
        .insert_resource(ClearColor(
            Color::rgb(
                38.0 / 255.0, 
                28.0 / 255.0, 
                37.0 / 255.0
            )
        ))
        .insert_resource(BirdConfiguration {
            alignment: 1.0,
            cohesion: 1.0,
            seperation: 1.5,
            speed: 100.0,
            steer: 2.0,
            radius: 32.0,
            neighbour_radius: 90.0,
            seperation_radius: 50.0,
            birds_amount: 500,
        })
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_birds)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(PHYSICS_STEP as f64))
                .with_system(cohesion_system)
                .with_system(seperation_system)
                .with_system(alignment_system)
                .with_system(movement_system)
                .with_system(wrapping_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(ANIMATE_STEP as f64))
                .with_system(sprite_animate_system)
        )
        .add_system(sprite_flip_x_system)
        .add_system(sprite_z_layer_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Bird;

#[derive(Resource)]
struct BirdConfiguration {
    alignment: f32,
    cohesion: f32,
    seperation: f32,
    neighbour_radius: f32,
    seperation_radius: f32,
    speed: f32,
    steer: f32,
    radius: f32,
    birds_amount: usize,
}

fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default())
        .insert(Name::new("Camera"));
}

fn spawn_birds(
    assets: Res<AssetServer>,
    configuration: Res<BirdConfiguration>,
    windows: Res<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands
) {
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width() / 2.0, window.height() / 2.0);

    let bird_handle = assets.load("birb.png");
    let mut atlas_handles = Vec::with_capacity(3);

    for i in 0..3 {
        let sprite_offset = Some(Vec2::new(0.0, i as f32 * 64.0));
        let texture_atlas = TextureAtlas::from_grid(bird_handle.clone(), Vec2::splat(64.0),  3,  1, None, sprite_offset);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        atlas_handles.push(texture_atlas_handle);
    }

    let mut rng = thread_rng();
    for _ in 0..configuration.birds_amount {
        let texture_atlas = atlas_handles[rng.gen_range(0..=2)].clone();
        commands.spawn(SpriteSheetBundle {
                texture_atlas,
                sprite: TextureAtlasSprite {
                    index: rng.gen_range(0..=2),
                    ..default()
                },
                transform: Transform::from_translation(
                    Vec3::new(
                        rng.gen_range(-width..=width), 
                        rng.gen_range(-height..=height),
                        1.0,
                    )
                ),
                ..default()
            })
            .insert(Velocity(
                Vec2::new(
                    rng.gen_range(-1.0..=1.0), 
                    rng.gen_range(-1.0..=1.0)
                )
            ))
            .insert(Bird)
            .insert(Name::new("Bird"));
    }
}

fn movement_system(
    mut query: Query<(&mut Transform, &Velocity)>
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * PHYSICS_STEP).extend(0.0);
    }
}

fn sprite_flip_x_system(
    mut query: Query<(&mut TextureAtlasSprite, &Velocity)>
) {
    for (mut sprite, velocity) in query.iter_mut() {
        sprite.flip_x = velocity.x > 0.0;
    }
}

fn sprite_animate_system(    
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>)>
) {
    for (mut sprite, handle) in query.iter_mut() {
        let texture_atlas = texture_atlases.get(handle).unwrap();
        sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
    }
}

fn sprite_z_layer_system(
    windows: Res<Windows>,
    camera: Query<&Camera>,
    mut query: Query<&mut Transform, With<TextureAtlasSprite>>
) {
    let camera = camera.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    let height = window.height() / 2.0;

    for mut transform in query.iter_mut() {
        transform.translation.z = (-transform.translation.y + height) / 100.0;
    }
}

fn wrapping_system(
    windows: Res<Windows>,
    configuration: Res<BirdConfiguration>,
    camera: Query<&Camera>,
    mut query: Query<&mut Transform, With<Bird>>
) {
    let camera = camera.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    let (width, height) = (window.width() / 2.0, window.height() / 2.0);

    for mut transform in query.iter_mut() {
        if transform.translation.x - configuration.radius > width {
            transform.translation.x = -width - configuration.radius;
        }
        else if transform.translation.x + configuration.radius < -width {
            transform.translation.x = width + configuration.radius;
        }
        if transform.translation.y - configuration.radius > height {
            transform.translation.y = -height - configuration.radius;
        }
        else if transform.translation.y + configuration.radius < -height {
            transform.translation.y = height + configuration.radius;
        }
    }
}

fn cohesion_system(
    configuration: Res<BirdConfiguration>,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Bird>>
) {
    let birds = query
        .into_iter()
        .map(|(entity, transform, _)| (entity.index(), transform.translation.truncate()) )
        .collect::<Vec<(u32, Vec2)>>();

    for (entity, transform, mut velocity) in query.iter_mut() {
        let mut count = 0;
        let mut sum = Vec2::ZERO;

        for (other_entity, other_position) in birds.iter() {
            if other_entity == &entity.index() {
                continue;
            }

            let distance = transform.translation.truncate().distance(*other_position);

            if distance < configuration.neighbour_radius {
                sum += *other_position;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            let mut desired = sum - transform.translation.truncate();
            desired = desired.normalize();
            desired *= configuration.speed;

            let mut steer = desired - velocity.0;
            if steer.length() > configuration.steer {
                steer = steer.normalize() * configuration.steer;
            }

            velocity.0 += steer * configuration.cohesion;
        }
    }
}

fn seperation_system(
    configuration: Res<BirdConfiguration>,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Bird>>
) {
    let birds = query
        .into_iter()
        .map(|(entity, transform, _)| (entity.index(), transform.translation.truncate()) )
        .collect::<Vec<(u32, Vec2)>>();

    for (entity, transform, mut velocity) in query.iter_mut() {
        let mut count = 0;
        let mut sum = Vec2::ZERO;

        for (other_entity, other_position) in birds.iter() {
            if other_entity == &entity.index() {
                continue;
            }
            let position = transform.translation.truncate();
            let distance = position.distance(*other_position);

            if distance < configuration.seperation_radius {
                let mut difference = position - *other_position;
                difference = difference.normalize();
                difference /= distance;
                sum += difference;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
        }

        if sum.length() > 0.0 {
            sum = sum.normalize();
            sum *= configuration.speed;
            sum -= velocity.0;
            if sum.length() > configuration.steer {
                sum = sum.normalize() * configuration.steer;
            }
            velocity.0 += sum * configuration.seperation;
        }
    }
}

fn alignment_system(
    configuration: Res<BirdConfiguration>,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Bird>>
) {
    let birds = query
        .into_iter()
        .map(|(entity, transform, velocity)| (entity.index(), transform.translation.truncate(), velocity.0) )
        .collect::<Vec<(u32, Vec2, Vec2)>>();

    for (entity, transform, mut velocity) in query.iter_mut() {
        let mut count = 0;
        let mut sum = Vec2::ZERO;

        for (other_entity, other_position, other_velocity) in birds.iter() {
            if other_entity == &entity.index() {
                continue;
            }
            let position = transform.translation.truncate();
            let distance = position.distance(*other_position);

            if distance < configuration.neighbour_radius {
                sum += *other_velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            sum = sum.normalize();
            sum *= configuration.speed;

            let mut steer = sum - velocity.0;
            if steer.length() > configuration.steer {
                steer = steer.normalize() * configuration.steer;
            }

            velocity.0 += steer * configuration.alignment;
        }
    }
}