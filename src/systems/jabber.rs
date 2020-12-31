use bevy::prelude::*;
use rand::Rng;

const LINES_OF_GREAT_IMPORTANCE: [&'static str; 7] = [
    "This isn't just about you. It's about what's best for all of us.",
    "What are you gonna do?",
    "Think about it. Is it really the right choice?",
    "There is no way back after that...",
    "Put it behind you! There are greater things at stake.",
    "Let's go through it again, honestly.",
    "This puts us back to square one.",
];

const LINES_OF_COOKING: [&'static str; 10] = [
    "More cheese...",
    "More salt...",
    "Something is missing...",
    "Some honey...",
    "Bunch of chillies...",
    "Now the vinegar...",
    "Tastes good already...",
    "Just salt and pepper a bit...",
    "Was it too much?",
    "Oh no, thats too much.",
];

pub struct JabberingTimer(Timer);

pub struct Jabbering {
    pub line: Option<usize>,
    pub lines: &'static [&'static str],
}

impl Jabbering {
    pub fn get_line(&self) -> Option<String> {
        for index in self.line {
            for line in self.lines.get(index) {
                return Some(line.to_string());
            }
        }
        None
    }
}

pub fn jabbering_system(time: Res<Time>, mut query: Query<(Mut<JabberingTimer>, Mut<Jabbering>)>) {
    let mut rng = rand::thread_rng();

    for (mut timer, mut jabbering) in query.iter_mut() {
        if timer.0.tick(time.delta_seconds()).just_finished() {
            let lines = jabbering.lines;
            jabbering.line = Some(rng.gen_range(0, lines.len()));
        }
    }
}

pub fn print_jabbering_system(
    query: Query<(Entity, &Jabbering, Option<&String>), Changed<Jabbering>>,
) {
    for (entity, jabbering, name) in query.iter() {
        for line in jabbering.get_line() {
            if let Some(name) = name {
                println!("{}: {}", name, line);
            } else {
                println!("Entity {}: {}", entity.id(), line);
            }
        }
    }
}

pub struct RenderedJabbering(Entity);
pub struct RenderedJabberingRoot;

pub fn rendered_jabbering_system(
    font: Res<Handle<Font>>,
    commands: &mut Commands,
    query: Query<
        (
            Entity,
            &Jabbering,
            Option<&String>,
            Option<&RenderedJabbering>,
        ),
        Changed<Jabbering>,
    >,
    mut text_query: Query<Mut<Text>>,
    root_query: Query<Entity, With<RenderedJabberingRoot>>,
) {
    let style = TextStyle {
        font_size: 30.0,
        color: Color::BLACK,
        alignment: TextAlignment {
            vertical: VerticalAlign::Top,
            horizontal: HorizontalAlign::Left,
        },
    };

    let root = root_query.iter().next();
    if root.is_none() {
        return;
    }
    let root = root.unwrap();

    for (entity, jabbering, name, rendered) in query.iter() {
        for line in jabbering.get_line() {
            let mut text_value = if let Some(name) = name {
                format!("{}: ", name)
            } else {
                format!("Entity {}: ", entity.id())
            };
            text_value.push_str(&line);

            if let Some(child) = rendered {
                if let Ok(mut text_comp) = text_query.get_mut(child.0) {
                    text_comp.value = text_value;
                }
            } else {
                let child = commands
                    .spawn(TextBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Percent(50.0)),
                            ..Default::default()
                        },
                        text: Text {
                            value: text_value,
                            font: font.clone(),
                            style: style.clone(),
                        },
                        ..Default::default()
                    })
                    .current_entity()
                    .unwrap();

                commands.insert_one(entity, RenderedJabbering(child));
                commands.push_children(root, &[child]);
            }
        }
    }
}

pub fn example() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .add_system(jabbering_system.system())
        .add_system_to_stage(stage::POST_UPDATE, print_jabbering_system.system())
        .add_system_to_stage(stage::UPDATE, rendered_jabbering_system.system())
        .run();
}

fn example_setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands.spawn(CameraUiBundle::default());
    commands.insert_resource::<Handle<Font>>(asset_server.load("FiraSans-Bold.ttf"));

    let mut timer1 = Timer::from_seconds(3.0, true);
    let timer2 = timer1.clone();
    timer1.set_elapsed(1.5);

    commands.spawn((
        "Chef".to_string(),
        Jabbering {
            line: None,
            lines: &LINES_OF_COOKING,
        },
        JabberingTimer(timer1),
    ));

    commands.spawn((
        "Bob".to_string(),
        Jabbering {
            line: None,
            lines: &LINES_OF_GREAT_IMPORTANCE,
        },
        JabberingTimer(timer2),
    ));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(RenderedJabberingRoot);
}
