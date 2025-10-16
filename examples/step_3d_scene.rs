//! A 3D scene example that loads and displays STEP files from the asset server
//! Based on the default Bevy 3D scene example
use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (load_step_models, rotate_models))
        .run();
}

/// Set up a simple 3D scene with STEP models
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load STEP files from the asset server
    let step_handle_1: Handle<StepAsset> = asset_server.load("22604_bcab4db9_0001_2.step");
    let step_handle_2: Handle<StepAsset> = asset_server.load("76879_65a30a82_0010_2.step");

    // Spawn STEP models when they're loaded
    commands.spawn((
        StepModelLoader {
            handle: step_handle_1,
            position: Vec3::new(-30.0, 0.0, 0.0),
            colour: Color::srgb(0.8, 0.7, 0.6),
            name: "Model 1".to_string(),
        },
        Name::new("STEP Model Loader 1"),
    ));

    commands.spawn((
        StepModelLoader {
            handle: step_handle_2,
            position: Vec3::new(30.0, 0.0, 0.0),
            colour: Color::srgb(0.6, 0.7, 0.8),
            name: "Model 2".to_string(),
        },
        Name::new("STEP Model Loader 2"),
    ));

    // Circular base for reference
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Name::new("Base"),
    ));

    // Enhanced lighting setup for better model illumination
    // Multiple point lights positioned at 45° angles from camera
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 1500000.0, // Very bright to illuminate STEP models well
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(7.0, 7.0, 7.0), // 45° angle from camera position
        Camera {
            order: 0, // Same order as main camera
            ..default()
        },
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 1200000.0, // Secondary light with slightly lower intensity
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(-7.0, 7.0, -7.0), // 45° angle from camera position
        Camera {
            order: 0, // Same order as main camera
            ..default()
        },
    ));
    
    // Ambient light for overall scene illumination
    commands.spawn(AmbientLight {
        color: Color::srgb(0.2, 0.2, 0.2), // Soft white ambient light
        brightness: 150.0,
        affects_lightmapped_meshes: false, // Don't affect lightmapped meshes
    });

    // Orthographic camera - moved back significantly to accommodate larger models
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_xyz(0.0, 0.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Camera"),
    ));

    println!("Scene initialised. Loading STEP models from asset server...");
}

#[derive(Component)]
struct StepModelLoader {
    handle: Handle<StepAsset>,
    position: Vec3,
    colour: Color,
    name: String,
}

#[derive(Component)]
struct RotatingStepModel;

fn load_step_models(
    _time: Res<Time>,
    step_assets: Res<Assets<StepAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &StepModelLoader), Without<RotatingStepModel>>,
    mut commands: Commands,
) {
    for (entity, loader) in &mut query {
        if let Some(step_asset) = step_assets.get(&loader.handle) {
            println!("✅ Loaded STEP model: {}", loader.name);
            
            // Create material for this STEP model
            let material = materials.add(StandardMaterial {
                base_color: loader.colour,
                metallic: 0.1,
                perceptual_roughness: 0.5,
                ..default()
            });

            // Spawn the STEP model with unit scaling and significant spacing
            commands.spawn((
                Mesh3d(meshes.add(step_asset.mesh.clone())),
                MeshMaterial3d(material),
                Transform::from_translation(loader.position)
                    .with_scale(Vec3::splat(0.3)), // Much smaller scale to make models unit size
                GlobalTransform::default(),
                RotatingStepModel,
                Name::new(format!("Loaded {}", loader.name)),
            ));

            // Remove the loader component to prevent re-execution
            commands.entity(entity).remove::<StepModelLoader>();
            
            println!("  - Spawned at position: {:?}", loader.position);
        }
    }
}

fn rotate_models(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RotatingStepModel>>,
) {
    for mut transform in &mut query {
        // Rotate about the Y axis (vertical axis) - this is typically the longest axis for most models
        transform.rotate_y(time.delta_secs() * 0.5);
        // Also add a slight rotation about X axis for better visualisation
        transform.rotate_x(time.delta_secs() * 0.2);
    }
}