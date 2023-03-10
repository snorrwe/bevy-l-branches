use bevy::{ecs::query::QueryEntityError, prelude::*, window::WindowMode};

pub const LAUNCHER_TITLE: &str = "L-branches";

#[derive(Component)]
struct Vertex {
    pub id: u32,
}

#[derive(Resource)]
struct NextId(pub u32);

#[derive(Resource)]
struct Spline {
    nodes: Vec<(u32, Vec3)>,
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

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("topic.png"),
            transform: Transform::from_translation(Vec3::new(
                -200.0, -50.0, 0.0,
            )),
            ..default()
        })
        .insert(Vertex { id: 0 });
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("topic.png"),
            transform: Transform::from_translation(Vec3::new(200.0, 50.0, 0.0)),
            ..default()
        })
        .insert(Vertex { id: 1 });
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("topic.png"),
            transform: Transform::from_translation(Vec3::new(
                150.0, 250.0, 0.0,
            )),
            ..default()
        })
        .insert(Vertex { id: 2 });

    next_id.0 = 3;
}

fn update_spline(
    q: Query<(&GlobalTransform, &Vertex)>,
    change: Query<(), Changed<Vertex>>,
    mut spline: ResMut<Spline>,
) {
    // if any vertex changes update the whole spline
    if change.iter().next().is_none() {
        return;
    }
    info!("Updating spline");
    spline.nodes.clear();
    spline
        .nodes
        .extend(q.iter().map(|(tr, v)| (v.id, tr.translation())));
    spline.nodes.sort_unstable_by_key(|(id, _)| *id);
}
