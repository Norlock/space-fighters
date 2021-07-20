use crate::{Enemy, Materials, Speed, WinSize, SCALE, TIME_STEP};
use bevy::{core::FixedTimestep, prelude::*};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(enemy_spawn.system()),
        )
        //.add_system_set(
        //SystemSet::new()
        //.with_run_criteria(FixedTimestep::step(1.0))
        //.with_system(enemy_movement.system()),
        //)
        .add_system(enemy_movement.system());
    }
}

fn enemy_spawn(mut commands: Commands, win_size: Res<WinSize>, materials: Res<Materials>) {
    // compute the random position
    let x = 0.;
    let y = 0.;

    // spawn enemy
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.enemy.clone(),
            transform: Transform {
                translation: Vec3::new(x, y, 10.0),
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy)
        .insert(Speed::default());
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
