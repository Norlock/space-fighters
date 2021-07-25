use crate::{
    ActiveEnemies, Enemy, EnemyLaser, Materials, PlayerState, Speed, WinSize, MAX_ENEMIES, SCALE,
    TIME_STEP,
};
use bevy::{core::FixedTimestep, prelude::*};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(enemy_spawn.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.9))
                .with_system(enemy_fire.system()),
        )
        .add_system(enemy_movement.system())
        .add_system(enemy_laser_movement.system());
    }
}

fn enemy_spawn(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut active_enemies: ResMut<ActiveEnemies>,
    materials: Res<Materials>,
) {
    if MAX_ENEMIES <= active_enemies.0 {
        return;
    }

    // compute the random position
    let x = -(win_size.width / 2.) + 50.;
    let y = (win_size.height / 2.) - 50.;

    // spawn enemy
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.enemy.clone(),
            transform: Transform {
                translation: Vec3::new(x, y, 10.),
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy)
        .insert(Speed::default());

    active_enemies.0 += 1;
}

fn enemy_movement(mut query: Query<(&Speed, &mut Transform), With<Enemy>>, win_size: Res<WinSize>) {
    for (speed, mut transform) in query.iter_mut() {
        if transform.translation.x < win_size.width {
            transform.translation.x += 0.3 * speed.0 * TIME_STEP;
        } else {
            transform.translation.x = 0.;
            transform.translation.y -= 64.;
        }
    }
}

fn enemy_fire(
    mut commands: Commands,
    materials: Res<Materials>,
    player_state: Res<PlayerState>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    if !player_state.on {
        return;
    }

    for &tf in enemy_query.iter() {
        let x = tf.translation.x;
        let y = tf.translation.y;
        // Spawn enemy laser sprite
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.enemy_laser.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y - 15., 0.),
                    scale: Vec3::new(SCALE, -SCALE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(EnemyLaser)
            .insert(Speed::default());
    }
}

fn enemy_laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut laser_query: Query<(Entity, &Speed, &mut Transform), With<EnemyLaser>>,
) {
    for (entity, speed, mut tf) in laser_query.iter_mut() {
        tf.translation.y -= speed.0 * TIME_STEP;
        tf.translation.x += 1.;
        if tf.translation.y < -win_size.height / 2. - 50. {
            commands.entity(entity).despawn();
        }
    }
}
