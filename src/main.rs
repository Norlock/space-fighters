mod enemy;
mod player;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use std::collections::HashSet;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";

const SCALE: f32 = 0.5;
const TIME_STEP: f32 = 1. / 60.;
const PLAYER_RESPAWN_DELAY: f64 = 2.;
const MAX_ENEMIES: u32 = 20;
const HORIZONTAL_MARGIN: f32 = 50.;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    Paused,
    Gameover,
}

// Entity, Component, System, Resource

// region:    Resources
pub struct Materials {
    player: Handle<ColorMaterial>,
    player_laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    enemy_laser: Handle<ColorMaterial>,
    explosion: Handle<TextureAtlas>,
}

struct WinSize {
    #[allow(unused)]
    width: f32,
    height: f32,
}

// endregion: Resources

// region:    Components
struct ActiveEnemies(u32);
struct LivesLeft(u32);
struct EnemiesLeft(u32);

struct PlayerState {
    on: bool,
    last_shot: f64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: 0.,
        }
    }
}

impl PlayerState {
    fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }

    fn spawned(&mut self) {
        self.on = true;
        self.last_shot = 0.;
    }
}

struct Player;
struct PlayerReadyFire(bool);

struct PlayerLaser;
struct EnemyLaser;

struct Enemy;

struct Explosion;
struct ExplosionToSpawn(Vec3);

// Text labels
struct LivesLeftText;
struct EnemiesLeftText;

struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500.)
    }
}
// endregion: Components

fn main() {
    App::build()
        .add_state(AppState::InGame)
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Space Fighters!".to_string(),
            width: 1000.,
            height: 800.,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0))
        .insert_resource(EnemiesLeft(100))
        .insert_resource(LivesLeft(5))
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(init_ui.system())
        .add_system(lives_left_system.system())
        .add_system(enemies_left_system.system())
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();

    // camera
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation.x = window.width() / 2.;
    camera.transform.translation.y = window.height() / 2.;
    commands.spawn_bundle(camera);

    // create the main resources
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    commands.insert_resource(Materials {
        player: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        player_laser: materials.add(asset_server.load(PLAYER_LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        enemy_laser: materials.add(asset_server.load(ENEMY_LASER_SPRITE).into()),
        explosion: texture_atlases.add(texture_atlas),
    });

    commands.insert_resource(WinSize {
        width: window.width(),
        height: window.height(),
    });

    // position window
    window.set_position(IVec2::new(0, 0));
}

fn player_laser_hit_enemy(
    mut commands: Commands,
    mut app_state: ResMut<State<AppState>>,
    laser_query: Query<(Entity, &Transform, &Sprite), With<PlayerLaser>>,
    enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    mut active_enemies: ResMut<ActiveEnemies>,
    mut enemies_left: ResMut<EnemiesLeft>,
) {
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();

    for (laser_entity, laser_tf, laser_sprite) in laser_query.iter() {
        for (enemy_entity, enemy_tf, enemy_sprite) in enemy_query.iter() {
            let laser_scale = Vec2::from(laser_tf.scale);
            let enemy_scale = Vec2::from(enemy_tf.scale);
            let collision = collide(
                laser_tf.translation,
                laser_sprite.size * laser_scale,
                enemy_tf.translation,
                enemy_sprite.size * enemy_scale,
            );

            if let Some(_) = collision {
                if enemies_blasted.get(&enemy_entity).is_none() {
                    // remove the enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;

                    enemies_left.0 -= 1;
                    if enemies_left.0 == 0 {
                        // todo state for finished lvl/game
                        app_state.set(AppState::Gameover).unwrap();
                    }

                    // spawn explosion to spawn
                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));

                    enemies_blasted.insert(enemy_entity);
                }

                // remove the laser
                commands.entity(laser_entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut lives_left: ResMut<LivesLeft>,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &Sprite), With<EnemyLaser>>,
    player_query: Query<(Entity, &Transform, &Sprite), With<Player>>,
) {
    if let Ok((player_entity, player_tf, player_sprite)) = player_query.single() {
        let player_size = player_sprite.size * Vec2::from(player_tf.scale.abs());

        for (laser_entity, laser_tf, laser_sprite) in laser_query.iter() {
            let laser_size = laser_sprite.size * Vec2::from(laser_tf.scale.abs());

            // compute collision
            let collision = collide(
                laser_tf.translation,
                laser_size,
                player_tf.translation,
                player_size,
            );

            // process collision
            if let Some(_) = collision {
                // remove the player
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());
                // remove the laser
                commands.entity(laser_entity).despawn();

                lives_left.0 -= 1;
                if lives_left.0 == 0 {
                    app_state.set(AppState::Gameover).unwrap();
                }

                // spawn the explosion to spawn entity
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: materials.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<
        (
            Entity,
            &mut Timer,
            &mut TextureAtlasSprite,
            &Handle<TextureAtlas>,
        ),
        With<Explosion>,
    >,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;
            if sprite.index == texture_atlas.textures.len() as u32 {
                commands.entity(entity).despawn()
            }
        }
    }
}

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    // enemies left text
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Enemies left: 100".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(EnemiesLeftText);

    // lives left lext
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Lives left: 5".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LivesLeftText);
}

fn lives_left_system(
    lives_left: Res<LivesLeft>,
    mut lives_left_query: Query<&mut Text, With<LivesLeftText>>,
) {
    let mut lives_left_text = lives_left_query.single_mut().unwrap();
    lives_left_text.sections[0].value = format!("Enemies left: {}", lives_left.0);
}

fn enemies_left_system(
    enemies_left: Res<EnemiesLeft>,
    mut enemies_left_query: Query<&mut Text, With<EnemiesLeftText>>,
) {
    let mut enemies_left_text = enemies_left_query.single_mut().unwrap();
    enemies_left_text.sections[0].value = format!("Enemies left: {}", enemies_left.0);
}
