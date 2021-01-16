fn main() {
    app().run();
}

use bevy::{
    core::Timer, input::system::exit_on_esc_system, prelude::*, utils::AHashExt, utils::HashMap,
};
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
        .add_event::<PlayerEvent>()
        .add_system(player_input.system())
        .add_system(player_move.system())
        .add_system(player_animation.system())
        .add_system(sprite_animation_update.system());
    app
}

const LAYER_0: f32 = 500.0;

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
        Vec2::new(0.0, 0.0),
        &mut atlases,
        commands,
    );

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
        GlobalTransform::default(),
        Transform::from_translation(Vec3::new(0.0, 16.0, LAYER_0)),
        SpriteAnimation::new(
            dress,
            vec![("standing", vec![8]), ("walking", vec![0, 1, 2])],
        ),
    ));

    commands.insert_one(
        player,
        RigidBodyBuilder::new_dynamic()
            .lock_rotations()
            .user_data(player.to_user_data()),
    );

    let collider = commands.entity((ColliderBuilder::ball(2.0).user_data(player.to_user_data()),));

    commands.push_children(player, &[dress, collider]);
}

struct SpriteAnimation {
    current: Option<&'static str>,
    anim_index: usize,
    map: HashMap<&'static str, Vec<u32>>,
    timer: Timer,
    target: Entity,
}

impl SpriteAnimation {
    pub fn new(target: Entity, anims: Vec<(&'static str, Vec<u32>)>) -> Self {
        let current = anims.first().map(|e| e.0);
        let timer = Timer::from_seconds(0.11, true);
        let mut map = HashMap::with_capacity(anims.len());
        for anim in anims {
            map.insert(anim.0, anim.1);
        }
        Self {
            current,
            anim_index: 0,
            map,
            timer,
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

    pub fn update(&mut self, secs: f32) {
        if self.timer.tick(secs).finished() {
            self.anim_index = self.anim_index.wrapping_add(1);
        }
    }
}

fn sprite_animation_update(
    time: Res<Time>,
    mut anim_query: Query<Mut<SpriteAnimation>>,
    mut sprite_query: Query<Mut<TextureAtlasSprite>>,
) {
    for mut anim in anim_query.iter_mut() {
        anim.update(time.delta_seconds());

        for mut sprite in sprite_query.get_mut(anim.target) {
            sprite.index = anim.index();
        }
    }
}

struct PlayerMarker;

enum PlayerEvent {
    Interact,
    Observe,
    Move(Vec2),
}

fn player_input(keys: Res<Input<KeyCode>>, mut events: ResMut<Events<PlayerEvent>>) {
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
    if movement.x != 0.0 || movement.y != 0.0 {
        events.send(PlayerEvent::Move(movement.normalize()))
    } else {
        events.send(PlayerEvent::Move(movement))
    }

    if keys.just_pressed(KeyCode::E) {
        events.send(PlayerEvent::Interact);
    }

    if keys.just_pressed(KeyCode::F) {
        events.send(PlayerEvent::Observe);
    }
}

fn player_move(
    mut reader: Local<EventReader<PlayerEvent>>,
    events: Res<Events<PlayerEvent>>,
    mut bodies: ResMut<RigidBodySet>,
    player_query: Query<&RigidBodyHandleComponent, With<PlayerMarker>>,
) {
    for event in reader.iter(&events) {
        match event {
            PlayerEvent::Move(dir) => {
                let movement: Vec2 = *dir * 30.0;

                for body in player_query.iter() {
                    if let Some(body) = bodies.get_mut(body.handle()) {
                        body.set_linvel(movement.into_vector2(), true);
                    }
                }
            }
            _ => {}
        }
    }
}

fn player_animation(
    mut bodies: ResMut<RigidBodySet>,
    mut player_query: Query<(Mut<SpriteAnimation>, &RigidBodyHandleComponent), With<PlayerMarker>>,
) {
    for (mut anim, body) in player_query.iter_mut() {
        for body in bodies.get_mut(body.handle()) {
            if body.linvel().magnitude_squared() > 0.5 {
                anim.set("walking");
            } else {
                anim.set("standing");
            }
        }
    }
}