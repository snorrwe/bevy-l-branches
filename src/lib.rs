use std::hint::unreachable_unchecked;

use bevy::{prelude::*, window::WindowMode};

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

pub fn app(fullscreen: bool) -> App {
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
    .insert_resource(NextId(0))
    .insert_resource(Spline { nodes: vec![] })
    .add_startup_system(setup)
    .add_system(update_spline);
    app
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_id: ResMut<NextId>,
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
        Vec2::new(200.0, 150.0),
        texture.clone(),
    );
    spawn_next_topic(
        &mut commands,
        &mut next_id,
        Vec2::new(100.0, 250.0),
        texture.clone(),
    );
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

fn update_spline(
    q: Query<(&GlobalTransform, &Vertex)>,
    change: Query<(), Changed<Vertex>>,
    mut spline: ResMut<Spline>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    trees: Query<Entity, With<Tree>>,
) {
    // if any vertex changes update the whole spline
    if change.iter().next().is_none() {
        return;
    }
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
        for _ in 0..3 {
            system.next().unwrap();
        }
        let system = system.next().unwrap();

        let mut dir = (to.1 - from.1).normalize();
        let mut pos = from.1;
        let mut stack = Vec::with_capacity(128);
        // TODO: step should be based on the distance of topics
        let step = to.1.distance(from.1) / 15.0;
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
                'F' => pos += dir * step,
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
                                transform: Transform::from_translation(
                                    pos.extend(0.0),
                                ),
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
