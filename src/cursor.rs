/*
use bevy::prelude::*;

#[derive(Default)]
struct CursorPlugin(Entity);

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(MyCursorState {
            cursor: Default::default(),
            camera_e: self.0,
        })
        .add_system(cursor_system);
    }
}

struct MyCursorState {
    cursor: EventReader<CursorMoved>,
    // need to identify the main camera
    camera_e: Entity,
}

fn cursor_system(
    mut state: ResMut<MyCursorState>,
    ev_cursor: Res<Events<CursorMoved>>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera components
    q_camera: Query<&Transform>,
) {
    let camera_transform = q_camera.get(state.camera_e).unwrap();

    for ev in state.cursor.iter(&ev_cursor) {
        // get the size of the window that the event is for
        let wnd = wnds.get(ev.id).unwrap();
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = ev.position - size / 2.0;

        // apply the camera transform
        let pos_world = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        let pos = Vec2::new(pos_world.x(), pos_world.y());
        eprintln!("World coords: {}/{}", pos.x, pos.y);
    }
}
*/