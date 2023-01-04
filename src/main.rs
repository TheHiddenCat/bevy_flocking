use std::f32::consts::PI;

use bevy::{
    prelude::*, 
    time::FixedTimestep, 
    render::camera::RenderTarget,
    tasks::available_parallelism,
};

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
                .with_system(flock_system)
                .with_system(movement_system)
                .with_system(wrapping_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(ANIMATE_STEP as f64))
                .with_system(sprite_animate_system)
                .with_system(sprite_flip_x_system)
                .with_system(sprite_z_layer_system)
        )
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
    commands.spawn(Camera2dBundle::default()).insert(Name::new("Camera"));
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
        let angle = rng.gen_range(0.0..PI*2.0);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * configuration.speed;
        let position = Vec3::new(rng.gen_range(-width..=width), rng.gen_range(-height..=height), 1.0);
        let index = rng.gen_range(0..=2);

        commands.spawn(SpriteSheetBundle {
                texture_atlas,
                sprite: TextureAtlasSprite {
                    index,
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            })
            .insert(Velocity(velocity))
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

fn flock_system(
    configuration: Res<BirdConfiguration>,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Bird>>
) {
    let birds = query
        .into_iter()
        .map(|(entity, transform, velocity)| (entity.index(), transform.translation.truncate(), velocity.0) )
        .collect::<Vec<(u32, Vec2, Vec2)>>();

    query.par_for_each_mut(available_parallelism(), |(entity, transform, mut velocity)| {
        let mut count = 0;
        let mut too_close = 0;
        let mut cohesion = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut seperation = Vec2::ZERO;

        let position = transform.translation.truncate();

        for (other_entity, other_position, other_velocity) in birds.iter() {
            if other_entity == &entity.index() {
                continue;
            }
            let distance = position.distance(*other_position);
            if distance < configuration.neighbour_radius {
                cohesion += *other_position;
                alignment += *other_velocity;
                count += 1;
            }

            if distance < configuration.seperation_radius {
                let mut difference = position - *other_position;
                difference = difference.normalize() / distance;
                seperation += difference;
                too_close += 1;
            }
        }

        if count > 0 {
            cohesion /= count as f32;
            cohesion -= position;
            cohesion = cohesion.normalize() * configuration.speed;
            cohesion -= velocity.0;
            if cohesion.length() > configuration.steer {
                cohesion = cohesion.normalize() * configuration.steer;
            }
            cohesion *= configuration.cohesion;

            alignment /= count as f32;
            alignment = alignment.normalize() * configuration.speed;
            alignment -= velocity.0;
            if alignment.length() > configuration.steer {
                alignment = alignment.normalize() * configuration.steer;
            }
            alignment *= configuration.alignment;
        }

        if too_close > 0 {
            seperation /= too_close as f32;
        }

        if seperation.length() > 0.0 {
            seperation = seperation.normalize() * configuration.speed;
            seperation -= velocity.0;
            if seperation.length() > configuration.steer {
                seperation = seperation.normalize() * configuration.steer;
            }
            seperation *= configuration.seperation;
        }

        velocity.0 += cohesion + alignment + seperation;
    });
}