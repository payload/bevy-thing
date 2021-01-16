fn main() {
    app().run();
}

use bevy::{core::Timer, input::system::exit_on_esc_system, prelude::*, utils::HashMap};
use bevy_thing::{
    bevy_rapier_utils::*, commands_ext::CommandsExt, systems::texture_atlas_utils::*,
};

fn app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //
        .add_plugin(RapierPhysicsPlugin)
        // .add_plugin(RapierRenderPlugin)
        .add_plugin(TextureAtlasUtilsPlugin)
        //
        .add_system(exit_on_esc_system.system())
        //
        .add_startup_system(setup.system())
        .add_system(player_input.system())
        .add_system(player_update.system())
        .add_system(player_animation.system())
        .add_system(sprite_animation_update.system())
        .add_system(y_sort.system())
        .add_system(handle_actions.system())
        .add_event::<Action>();
    app
}

const LAYER_0: f32 = 500.0;
const LAYER_10: f32 = LAYER_0 + 10.0;

struct HumanAtlas(Handle<TextureAtlas>);
struct OvenAtlas(Handle<TextureAtlas>);

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut rapier: ResMut<RapierConfiguration>,
    mut clear_color: ResMut<ClearColor>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    rapier.gravity.y = 0.0;
    clear_color.0 = Color::rgb(0.133, 0.137, 0.137);

    let human_tex = asset_server.load("human.png");
    let human_atlas = texture_atlas_grid(
        human_tex.clone(),
        Vec2::new(8.0, 8.0),
        Vec2::zero(),
        &mut atlases,
        commands,
    );

    let oven_tex = asset_server.load("oven.png");
    let oven_atlas = texture_atlas_grid(
        oven_tex.clone(),
        Vec2::new(8.0, 8.0),
        Vec2::zero(),
        &mut atlases,
        commands,
    );

    commands.insert_resource(HumanAtlas(human_atlas.clone()));
    commands.insert_resource(OvenAtlas(oven_atlas.clone()));

    commands.spawn({
        let mut cam = Camera2dBundle::default();
        cam.transform.scale.x = 0.125;
        cam.transform.scale.y = 0.125;
        cam
    });

    //

    let dress = commands.entity(SpriteSheetBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
        texture_atlas: human_atlas.clone(),
        sprite: TextureAtlasSprite::new(0),
        ..Default::default()
    });

    let player = commands.entity((
        "Player".to_string(),
        PlayerMarker,
        YSortMarker,
        PlayerState::Idle,
        Transform::from_translation(Vec3::new(0.0, 16.0, LAYER_0)),
        GlobalTransform::default(),
        SpriteAnimation::new(dress, &[("standing", &[8]), ("walking", &[0, 17, 26])]),
    ));

    commands.insert_one(
        player,
        RigidBodyBuilder::new_dynamic()
            .translation(0.0, 16.0)
            .lock_rotations()
            .user_data(player.to_user_data()),
    );

    let collider = commands.entity((ColliderBuilder::ball(2.0).user_data(player.to_user_data()),));

    commands.push_children(player, &[dress, collider]);

    //

    let dress = commands.entity(SpriteSheetBundle {
        texture_atlas: oven_atlas.clone(),
        ..Default::default()
    });
    let oven = commands.entity((
        "Oven".to_string(),
        YSortMarker,
        OvenMarker,
        SpriteAnimation::new(
            dress,
            &[
                ("off", &[0]),
                ("on", &[1]),
                ("on_fish", &[2]),
                ("off_fish", &[3]),
                ("on_bakedfish", &[4]),
                ("off_bakedfish", &[5]),
            ],
        ),
        Transform::from_translation(Vec3::new(16.0, 0.0, LAYER_0)),
        GlobalTransform::default(),
    ));
    commands.push_children(oven, &[dress]);
}

struct ItemMarker;

struct YSortMarker;

fn y_sort(mut trans_query: Query<Mut<Transform>, With<YSortMarker>>) {
    for mut trans in trans_query.iter_mut() {
        trans.translation.z = trans.translation.z.floor() + 0.5 - trans.translation.y * 0.001;
    }
}

struct SpriteAnimation {
    current: Option<&'static str>,
    anim_index: usize,
    map: HashMap<&'static str, Vec<u32>>,
    timer: Timer,
    flip_x: bool,
    target: Entity,
}

impl SpriteAnimation {
    pub fn new(target: Entity, anims: &[(&'static str, &[u32])]) -> Self {
        let current = anims.first().map(|e| e.0);
        let timer = Timer::from_seconds(0.11, true);
        let mut map = HashMap::default();
        map.reserve(anims.len());

        for anim in anims {
            map.insert(anim.0, anim.1.iter().cloned().collect());
        }

        Self {
            current,
            anim_index: 0,
            map,
            timer,
            flip_x: false,
            target,
        }
    }

    pub fn index(&self) -> u32 {
        for name in self.current {
            for anim in self.map.get(name) {
                return anim[self.anim_index % anim.len()];
            }
        }
        0
    }

    pub fn set(&mut self, name: &'static str) {
        self.current = Some(name);
    }

    pub fn get(&self) -> Option<&str> {
        self.current
    }

    pub fn update(&mut self, secs: f32) {
        if self.timer.tick(secs).finished() {
            self.anim_index = self.anim_index.wrapping_add(1);
        }
    }
}

fn sprite_animation_update(
    time: Res<Time>,
    mut anim_query: Query<Mut<SpriteAnimation>>,
    mut sprite_query: Query<(Mut<Transform>, Mut<TextureAtlasSprite>)>,
) {
    for mut anim in anim_query.iter_mut() {
        anim.update(time.delta_seconds());

        for (mut trans, mut sprite) in sprite_query.get_mut(anim.target) {
            sprite.index = anim.index();

            if anim.flip_x && trans.scale.x > 0.0 {
                trans.scale.x = -trans.scale.x;
            } else if !anim.flip_x && trans.scale.x < 0.0 {
                trans.scale.x = -trans.scale.x;
            }
        }
    }
}

struct PlayerMarker;
struct OvenMarker;

#[derive(Clone, Debug, PartialEq)]
enum PlayerState {
    Idle,
    Interact,
    Observe,
    Move(Vec2),
}

fn player_input(keys: Res<Input<KeyCode>>, mut state_query: Query<Mut<PlayerState>>) {
    let mut movement = Vec2::default();
    if keys.pressed(KeyCode::W) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        movement.y -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        movement.x += 1.0;
    }
    let has_movement = if movement.x != 0.0 || movement.y != 0.0 {
        movement = movement.normalize();
        true
    } else {
        false
    };

    for mut state in state_query.iter_mut() {
        let new_state = match *state {
            PlayerState::Idle => {
                if keys.just_pressed(KeyCode::E) {
                    PlayerState::Interact
                } else if keys.just_pressed(KeyCode::F) {
                    PlayerState::Observe
                } else if has_movement {
                    PlayerState::Move(movement)
                } else {
                    PlayerState::Idle
                }
            }
            PlayerState::Interact => {
                if keys.just_released(KeyCode::E) {
                    PlayerState::Idle
                } else {
                    PlayerState::Interact
                }
            }
            PlayerState::Observe => {
                if keys.just_released(KeyCode::F) {
                    PlayerState::Idle
                } else {
                    PlayerState::Observe
                }
            }
            PlayerState::Move(_) => {
                if keys.just_pressed(KeyCode::E) {
                    PlayerState::Interact
                } else if has_movement {
                    PlayerState::Move(movement)
                } else {
                    PlayerState::Idle
                }
            }
        };

        if new_state != *state {
            *state = new_state;
        }
    }
}

fn player_update(
    mut actions: ResMut<Events<Action>>,
    mut bodies: ResMut<RigidBodySet>,
    player_query: Query<
        (Entity, &PlayerState, &Transform, &RigidBodyHandleComponent),
        (Changed<PlayerState>, With<PlayerMarker>),
    >,
    oven_query: Query<(Entity, &Transform), With<OvenMarker>>,
    mut anim_query: Query<Mut<SpriteAnimation>>,
) {
    for (player, state, trans, body) in player_query.iter() {
        match state {
            PlayerState::Move(dir) => {
                let movement: Vec2 = *dir * 30.0;
                for body in bodies.get_mut(body.handle()) {
                    body.set_linvel(movement.into_vector2(), true);
                }
            }
            _ => {
                for body in bodies.get_mut(body.handle()) {
                    body.set_linvel(Vector2::default(), true);
                }
            }
        }

        match state {
            PlayerState::Interact => {
                for (oven, oven_trans) in oven_query.iter() {
                    if pos(trans).distance_squared(pos(oven_trans)) < 64.0 {
                        // interact with oven

                        for mut anim in anim_query.get_mut(oven) {
                            let name = anim.get().map(String::from);
                            for name in name {
                                match name.as_str() {
                                    "off" => anim.set("on"),
                                    "on" => anim.set("on_fish"),
                                    "on_fish" => anim.set("off_fish"),
                                    "off_fish" => {
                                        anim.set("off");
                                        actions.send(Action::TransferItem("fish", oven, player));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

enum Action {
    TransferItem(&'static str, Entity, Entity),
}

fn handle_actions(
    commands: &mut Commands,
    transform_query: Query<&Transform>,
    mut reader: Local<EventReader<Action>>,
    actions: Res<Events<Action>>,
    oven_atlas: Res<OvenAtlas>,
) {
    for action in reader.iter(&actions) {
        match action {
            Action::TransferItem(item, from, to) => {
                for from_trans in transform_query.get(*from) {
                    for to_trans in transform_query.get(*to) {
                        let from_pos = pos(from_trans);
                        let _to_pos = pos(to_trans);

                        let dress = commands.entity(SpriteSheetBundle {
                            transform: Transform::from_xyz(0.0, 8.0, 0.0),
                            texture_atlas: oven_atlas.0.clone(),
                            ..Default::default()
                        });

                        let mut anim =
                            SpriteAnimation::new(dress, &[("fish", &[10]), ("bakedfish", &[11])]);
                        anim.set(item);

                        let item = commands.entity((
                            "Item".to_string(),
                            ItemMarker,
                            Transform::from_translation(from_pos.extend(LAYER_10)),
                            GlobalTransform::default(),
                            anim,
                        ));

                        commands.push_children(item, &[dress]);
                    }
                }
            }
        }
    }
}

fn player_animation(
    mut bodies: ResMut<RigidBodySet>,
    mut player_query: Query<(Mut<SpriteAnimation>, &RigidBodyHandleComponent), With<PlayerMarker>>,
) {
    for (mut anim, body) in player_query.iter_mut() {
        for body in bodies.get_mut(body.handle()) {
            let linvel = body.linvel();

            if linvel.magnitude_squared() > 0.5 {
                anim.set("walking");

                for x in linvel.get(0) {
                    if *x > 0.0 {
                        anim.flip_x = true;
                    }
                    if *x < 0.0 {
                        anim.flip_x = false;
                    }
                }
            } else {
                anim.set("standing");
            }
        }
    }
}

fn pos(trans: &Transform) -> Vec2 {
    trans.translation.truncate()
}
