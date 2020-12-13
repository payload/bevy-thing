use crate::stuff;

use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*
};

#[derive(Default)]
pub struct MyInputState {
    keys: EventReader<KeyboardInput>,
    cursor: EventReader<CursorMoved>,
    motion: EventReader<MouseMotion>,
    mousebtn: EventReader<MouseButtonInput>,
    scroll: EventReader<MouseWheel>,
}

macro_rules! debug {
    ($($arg:tt)*) => ({
        if false { eprintln!($($arg)*); }
    })
}

pub fn my_input_system(
    mut state: ResMut<MyInputState>,
    ev_keys: Res<Events<KeyboardInput>>,
    ev_cursor: Res<Events<CursorMoved>>,
    ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    ev_scroll: Res<Events<MouseWheel>>,
) {
    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            debug!("Just pressed key: {:?}", ev.key_code);
        } else {
            debug!("Just released key: {:?}", ev.key_code);
        }
    }

    // Absolute cursor position (in window coordinates)
    for ev in state.cursor.iter(&ev_cursor) {
        debug!("Cursor at: {}", ev.position);
    }

    // Relative mouse motion
    for ev in state.motion.iter(&ev_motion) {
        debug!("Mouse moved {} pixels", ev.delta);
    }

    // Mouse buttons
    for ev in state.mousebtn.iter(&ev_mousebtn) {
        if ev.state.is_pressed() {
            debug!("Just pressed mouse button: {:?}", ev.button);
        } else {
            debug!("Just released mouse button: {:?}", ev.button);
        }
    }

    // scrolling (mouse wheel, touchpad, etc.)
    for ev in state.scroll.iter(&ev_scroll) {
        debug!(
            "Scrolled vertically by {} and horizontally by {}.",
            ev.y, ev.x
        );
    }
}

#[test]
fn xsystem() {
    App::build().add_system(system);
}

pub fn system(
    game_state: Res<stuff::GameState>,
    time: Res<Time>,
    (keys, btns, windows): (Res<Input<KeyCode>>, Res<Input<MouseButton>>, Res<Windows>),
    mut transforms: Query<&mut Transform>,
) {
    let pressed = |code| if keys.pressed(code) { 1.0 } else { 0.0 };
    let x = pressed(KeyCode::D) - pressed(KeyCode::A);
    let y = pressed(KeyCode::W) - pressed(KeyCode::S);
    let s = pressed(KeyCode::Plus) - pressed(KeyCode::Minus);
    let r = pressed(KeyCode::E) - pressed(KeyCode::Q);
    if x != 0.0 || y != 0.0 || s != 0.0 || r != 0.0 {
        let vec = Vec3::new(x, y, 0.0);
        let scale_power = 1.1_f32.powf(-s);
        let scale = Vec3::new(scale_power, scale_power, 1.0);
        let trans = 40.0 * vec * time.delta_seconds();
        let angle = 3.14159 * 0.1 * r * time.delta_seconds();

        let mut transform = transforms
            .get_mut(game_state.camera)
            .expect("camera transform");
        transform.scale *= scale;
        transform.translation += trans * scale;
        transform.rotate(Quat::from_rotation_z(angle));
    }

    if btns.just_pressed(MouseButton::Left) {}

    if let Some(window) = windows.get_primary() {
        let pos = window.cursor_position();
        debug!("XXX {:?}", pos);
    }
}
