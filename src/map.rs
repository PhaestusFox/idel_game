use bevy::{asset::AsyncReadExt, prelude::*};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Cube>();
        app.register_asset_loader(Cube(Vec3::ZERO));
        app.add_systems(Startup, spawn_scene);
        app.add_systems(Update, add_changes);
    }
}

fn spawn_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Name::new("SceneRoot"),
        Mesh3d(assets.add(Mesh::from(Cuboid::from_length(1.0)))),
        MeshMaterial3d(assets.add(StandardMaterial {
            base_color: bevy::color::palettes::css::BLUE.into(),
            ..default()
        })),
        SceneRoot(assets.load("test.cube")),
    ));
}

#[derive(Component)]
struct SceneRoot(Handle<Cube>);

#[derive(Asset, Reflect, serde::Serialize, serde::Deserialize)]
struct Cube(Vec3);

impl bevy::asset::AssetLoader for Cube {
    type Asset = Self;
    type Error = &'static str;
    type Settings = ();

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        async move {
            let mut contents = String::new();
            if let Err(e) = reader.read_to_string(&mut contents).await {
                error!("{e:?}");
            };
            let cube = ron::from_str::<Vec3>(&contents).map_err(|e| {
                error!("Failed to parse cube asset: {:?}", e);
                "Failed to parse cube asset"
            })?;
            Ok(Cube(cube))
        }
    }
    fn extensions(&self) -> &[&str] {
        &["cube"]
    }
}

fn add_changes(
    mut changes: MessageReader<AssetEvent<Cube>>,
    cubes: Res<Assets<Cube>>,
    mut added: Query<(&mut Transform, Ref<SceneRoot>)>,
) {
    for change in changes.read() {
        match change {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(cube) = cubes.get(*id) {
                    for (mut transform, scene_root) in &mut added {
                        if scene_root.0.id() == *id {
                            transform.translation = cube.0;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (mut pos, root) in &mut added {
        if !root.is_changed() {
            continue;
        }
        let Some(cube) = cubes.get(root.0.id()) else {
            continue;
        };
        pos.translation = cube.0;
    }
}

#[test]
fn make_cube() {
    let cube = Cube(Vec3::ZERO);
    println!("{:?}", ron::to_string(&cube));
}
