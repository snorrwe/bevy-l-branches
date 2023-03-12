pub mod transport;

use std::hint::unreachable_unchecked;

use bevy::{prelude::*, window::WindowMode};
use bevy_prototype_lyon::prelude::*;
use transport::EventHandle;

pub const LAUNCHER_TITLE: &str = "L-branches";

#[derive(Component)]
struct Vertex {
    pub id: u32,
}

#[derive(Component)]
struct Leaf;

#[derive(Component)]
struct Tree;

#[derive(Resource)]
struct NextId(pub u32);

#[derive(Resource)]
struct Spline {
    nodes: Vec<(u32, Vec2)>,
}

struct SplineUpdate;

pub fn app(fullscreen: bool, events: EventHandle) -> App {
    let mode = if fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: LAUNCHER_TITLE.to_string(),
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            mode,
            ..default()
        }),
        ..default()
    }))
    .add_plugin(ShapePlugin)
    .add_plugin(transport::EventPlugin { handle: events })
    .insert_resource(NextId(0))
    .insert_resource(Spline { nodes: vec![] })
    .add_event::<SplineUpdate>()
    .add_startup_system(setup)
    // because of command resolution on_new_topic_msg has to come after update_spline,
    // so update_spline can see the new nodes when handling the events
    .add_system(on_new_topic_msg.after(update_spline))
    .add_system(update_spline);
    app
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_id: ResMut<NextId>,
    mut ev: EventWriter<SplineUpdate>,
) {
    commands.spawn(Camera2dBundle::default());

    let texture = asset_server.load("topic.png");
    spawn_next_topic(
        &mut commands,
        &mut next_id,
        Vec2::new(-200.0, -50.0),
        texture.clone(),
    );
    spawn_next_topic(
        &mut commands,
        &mut next_id,
        Vec2::new(300.0, 50.0),
        texture.clone(),
    );
    spawn_next_topic(
        &mut commands,
        &mut next_id,
        Vec2::new(100.0, 250.0),
        texture.clone(),
    );

    ev.send(SplineUpdate);
}

fn on_new_topic_msg(
    mut ev: EventReader<transport::ReceiveEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_id: ResMut<NextId>,
    mut update: EventWriter<SplineUpdate>,
) {
    let texture = asset_server.load("topic.png");
    for ev in ev.iter() {
        match ev.0 {
            transport::Event::AddTopic => {
                let pos = Vec2::new(-120.0, 89.0); // TODO: random pos
                spawn_next_topic(&mut commands, &mut next_id, pos, texture.clone());
                update.send(SplineUpdate);
            }
        }
    }
}

fn spawn_next_topic(
    commands: &mut Commands,
    next_id: &mut NextId,
    pos: Vec2,
    texture: Handle<Image>,
) {
    let id = next_id.0;
    next_id.0 += 1;

    commands
        .spawn(SpriteBundle {
            texture,
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        })
        .insert(Vertex { id });
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

fn update_spline(
    q: Query<(&GlobalTransform, &Vertex)>,
    mut spline: ResMut<Spline>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    trees: Query<Entity, With<Tree>>,
    mut ev: EventReader<SplineUpdate>,
) {
    // if any vertex changes update the whole spline
    if ev.iter().next().is_none() {
        return;
    }
    ev.clear();
    info!("Updating spline");
    spline.nodes.clear();
    spline
        .nodes
        .extend(q.iter().map(|(tr, v)| (v.id, tr.translation().truncate())));
    spline.nodes.sort_unstable_by_key(|(id, _)| *id);

    if spline.nodes.len() < 2 {
        return;
    }

    // clear leaves
    //
    for id in trees.iter() {
        commands.entity(id).despawn_recursive();
    }

    for win in spline.nodes.windows(2) {
        let [from, to] = win else {
            unsafe {unreachable_unchecked();}
        };
        let mut rules = lsystem::MapRules::new();
        rules.set_str('X', "F+[[X]-X]-F[-FX]+X");
        rules.set_str('F', "FF");
        let axiom = vec!['X'];
        let mut system = lsystem::LSystem::new(rules, axiom);

        // TODO: number of iterations should be based on the distance of the control points?
        for _ in 0..2 {
            system.next().unwrap();
        }
        let system = system.next().unwrap();

        let mut dir = (to.1 - from.1).normalize();
        let mut pos = Vec2::ZERO;
        let mut stack = Vec::with_capacity(128);
        // TODO: step should be based on the distance of topics
        let step = to.1.distance(from.1) / 20.0;
        const ANGLE: f32 = std::f32::consts::TAU / 8.0;

        let mut tree = commands.spawn((
            TransformBundle {
                local: Transform::from_translation(from.1.extend(0.0)),
                ..Default::default()
            },
            VisibilityBundle::default(),
            Tree,
        ));

        for c in system {
            match c {
                'F' => {
                    let new_pos = pos + dir * step;
                    let shape = bevy_prototype_lyon::shapes::Line(pos, new_pos);

                    let t = 1.0 - pos.distance_squared(from.1) / from.1.distance_squared(to.1);
                    pos = new_pos;

                    tree.with_children(move |commands| {
                        commands.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shape),
                                ..default()
                            },
                            Stroke::new(Color::BLACK, lerp(5.0, 10.0, t)),
                        ));
                    });
                }
                '-' => {
                    dir = Quat::from_rotation_z(ANGLE)
                        .mul_vec3(dir.extend(0.0))
                        .truncate()
                }
                '+' => {
                    dir = Quat::from_rotation_z(-ANGLE)
                        .mul_vec3(dir.extend(0.0))
                        .truncate()
                }
                '[' => {
                    stack.push((pos, dir));
                }
                ']' => {
                    // branch is done, insert a leaf
                    tree.with_children(|commands| {
                        commands
                            .spawn(SpriteBundle {
                                texture: asset_server.load("leaf.png"),
                                transform: Transform::from_translation(pos.extend(0.5)),
                                ..default()
                            })
                            .insert(Leaf);
                    });
                    (pos, dir) = stack.pop().unwrap();
                }
                'X' => {}
                _ => unreachable!("unexpected char {c}"),
            }
        }
    }
}
