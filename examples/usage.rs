use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            StepPlugin,
        ))
        .insert_resource(CameraState::default())
        .insert_resource(ModelPositions::default())
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(Update, (check_step_loaded, rotate_models, update_statistics, camera_control_system))
        .run();
}

#[derive(Resource)]
struct StepModel {
    handle: Handle<StepAsset>,
}

#[derive(Resource, Default)]
struct ModelPositions {
    positions: Vec<Vec3>,
}

#[derive(Resource)]
struct CameraState {
    zoom: f32,
    translation: Vec2,
    pan_start_pos: Option<Vec2>,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            zoom: 3.0,
            translation: Vec2::ZERO,
            pan_start_pos: None,
        }
    }
}

#[derive(Component)]
struct QuadrantLabel;

#[derive(Component)]
struct FoxtrotStatsDisplay;

#[derive(Component)]
struct OpenCascadeStatsDisplay;

#[derive(Component)]
struct FoxtrotSimplifiedStatsDisplay;

#[derive(Component)]
struct OpenCascadeSimplifiedStatsDisplay;

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the STEP file
    let step_handle = asset_server.load("22604_bcab4db9_0001_2.step");
    commands.insert_resource(StepModel { handle: step_handle });

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            near: 0.1,
            far: 2000.0,
            scale: 10.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 0,
            ..default()
        },
    ));

    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            range: 1500.0,
            ..default()
        },
        Transform::from_xyz(300.0, 400.0, 500.0),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 1200.0,
            shadows_enabled: true,
            range: 1500.0,
            ..default()
        },
        Transform::from_xyz(-300.0, 400.0, -500.0),
    ));
    
    commands.spawn(AmbientLight {
        color: Color::srgb(0.2, 0.2, 0.2),
        brightness: 150.0,
        affects_lightmapped_meshes: false,
    });
}

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
    ));

    // Labels for each quadrant with smaller font size
    commands.spawn((
        Text::new("Foxtrot"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.5, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(10),
            ..default()
        },
        QuadrantLabel,
    ));

    commands.spawn((
        Text::new("OCCT"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            right: px(10),
            ..default()
        },
        QuadrantLabel,
    ));

    commands.spawn((
        Text::new("Foxtrot"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10),
            left: px(10),
            ..default()
        },
        QuadrantLabel,
    ));

    commands.spawn((
        Text::new("OCCT"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10),
            right: px(10),
            ..default()
        },
        QuadrantLabel,
    ));

    commands.spawn((
        Text::new("verts: 0\ntris: 0\nedges: 0\nbytes: 0"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.5, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: px(50),
            left: px(10),
            ..default()
        },
        FoxtrotStatsDisplay,
    ));

    commands.spawn((
        Text::new("verts: 0\ntris: 0\nedges: 0\nbytes: 0"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: px(50),
            right: px(10),
            ..default()
        },
        OpenCascadeStatsDisplay,
    ));

    commands.spawn((
        Text::new("verts: 0\ntris: 0\nedges: 0\nbytes: 0\nw/meshopt"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(50),
            left: px(10),
            ..default()
        },
        FoxtrotSimplifiedStatsDisplay,
    ));

    commands.spawn((
        Text::new("verts: 0\ntris: 0\nedges: 0\nbytes: 0\nw/meshopt"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(50),
            right: px(10),
            ..default()
        },
        OpenCascadeSimplifiedStatsDisplay,
    ));
}

fn check_step_loaded(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut model_positions: ResMut<ModelPositions>,
    step_assets: Res<Assets<StepAsset>>,
    step_model: Option<Res<StepModel>>,
) {
    if let Some(step_model) = step_model {
        if let Some(step_asset) = step_assets.get(&step_model.handle) {
            println!("✅ STEP file loaded successfully!");
            
            let foxtrot_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.5, 0.5), // Light red
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            });
            
            let occt_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.9), // Light blue
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            });
            
            let foxtrot_simplified_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.9, 0.5), // Light green
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            });
            
            let occt_simplified_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.9, 0.5), // Light yellow
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            });

            // Scale models to unit size space them out..
            let scale_factor = 4.0;
            let spacing = 300.0;

            model_positions.positions.clear();

            // Top-left (Foxtrot): -spacing, spacing
            let foxtrot_pos = Vec3::new(-spacing, spacing, 0.0);
            model_positions.positions.push(foxtrot_pos);
            commands.spawn((
                Mesh3d(meshes.add(step_asset.mesh.clone())),
                MeshMaterial3d(foxtrot_material.clone()),
                Transform::from_xyz(foxtrot_pos.x, foxtrot_pos.y, foxtrot_pos.z)
                    .with_scale(Vec3::splat(scale_factor)),
                GlobalTransform::default(),
                RotatingModel,
                FoxtrotModel,
                ModelMetadata {
                    vertices: get_vertex_count(&step_asset.mesh),
                    triangles: get_triangle_count(&step_asset.mesh),
                    edges: get_triangle_count(&step_asset.mesh) * 3,
                },
            ));

            // Top-right (OpenCASCADE): spacing, spacing  
            let occt_pos = Vec3::new(spacing, spacing, 0.0);
            model_positions.positions.push(occt_pos);
            commands.spawn((
                Mesh3d(meshes.add(step_asset.mesh.clone())),
                MeshMaterial3d(occt_material.clone()),
                Transform::from_xyz(occt_pos.x, occt_pos.y, occt_pos.z)
                    .with_scale(Vec3::splat(scale_factor)),
                GlobalTransform::default(),
                RotatingModel,
                OpenCascadeModel,
                ModelMetadata {
                    vertices: get_vertex_count(&step_asset.mesh),
                    triangles: get_triangle_count(&step_asset.mesh),
                    edges: get_triangle_count(&step_asset.mesh) * 3,
                },
            ));

            // Bottom-left (Foxtrot + Simplification): -spacing, -spacing
            let foxtrot_simplified_pos = Vec3::new(-spacing, -spacing, 0.0);
            model_positions.positions.push(foxtrot_simplified_pos);
            #[cfg(feature = "meshopt")]
            {
                let mut simplified_asset = step_asset.clone();
                if simplified_asset.simplify_mesh(0.5, 0.01).is_ok() {
                    let vertices = get_vertex_count(&simplified_asset.mesh);
                    let triangles = get_triangle_count(&simplified_asset.mesh);
                    let edges = triangles * 3; 
                    
                    commands.spawn((
                        Mesh3d(meshes.add(simplified_asset.mesh)),
                        MeshMaterial3d(foxtrot_simplified_material.clone()),
                        Transform::from_xyz(foxtrot_simplified_pos.x, foxtrot_simplified_pos.y, foxtrot_simplified_pos.z)
                            .with_scale(Vec3::splat(scale_factor)),
                        GlobalTransform::default(),
                        RotatingModel,
                        FoxtrotSimplifiedModel,
                        ModelMetadata { vertices, triangles, edges },
                    ));
                } else {
                    // Fallback to original mesh if simplification fails
                    let vertices = get_vertex_count(&step_asset.mesh);
                    let triangles = get_triangle_count(&step_asset.mesh);
                    let edges = triangles * 3;
                    
                    commands.spawn((
                        Mesh3d(meshes.add(step_asset.mesh.clone())),
                        MeshMaterial3d(foxtrot_simplified_material.clone()),
                        Transform::from_xyz(foxtrot_simplified_pos.x, foxtrot_simplified_pos.y, foxtrot_simplified_pos.z)
                            .with_scale(Vec3::splat(scale_factor)),
                        GlobalTransform::default(),
                        RotatingModel,
                        FoxtrotSimplifiedModel,
                        ModelMetadata { vertices, triangles, edges },
                    ));
                }
            }
            #[cfg(not(feature = "meshopt"))]
            {
                let vertices = get_vertex_count(&step_asset.mesh);
                let triangles = get_triangle_count(&step_asset.mesh);
                let edges = triangles * 3; // Approximation
                
                commands.spawn((
                    Mesh3d(meshes.add(step_asset.mesh.clone())),
                    MeshMaterial3d(foxtrot_simplified_material.clone()),
                    Transform::from_xyz(foxtrot_simplified_pos.x, foxtrot_simplified_pos.y, foxtrot_simplified_pos.z)
                        .with_scale(Vec3::splat(scale_factor)),
                    GlobalTransform::default(),
                    RotatingModel,
                    FoxtrotSimplifiedModel,
                    ModelMetadata { vertices, triangles, edges },
                ));
            }

            // Bottom-right (OpenCASCADE + Simplification): spacing, -spacing
            let occt_simplified_pos = Vec3::new(spacing, -spacing, 0.0);
            model_positions.positions.push(occt_simplified_pos);
            #[cfg(all(feature = "opencascade", feature = "meshopt"))]
            {
                let mut simplified_asset = step_asset.clone();
                if simplified_asset.simplify_mesh(0.3, 0.01).is_ok() {
                    let vertices = get_vertex_count(&simplified_asset.mesh);
                    let triangles = get_triangle_count(&simplified_asset.mesh);
                    let edges = triangles * 3;
                    
                    commands.spawn((
                        Mesh3d(meshes.add(simplified_asset.mesh)),
                        MeshMaterial3d(occt_simplified_material.clone()),
                        Transform::from_xyz(occt_simplified_pos.x, occt_simplified_pos.y, occt_simplified_pos.z)
                            .with_scale(Vec3::splat(scale_factor)),
                        GlobalTransform::default(),
                        RotatingModel,
                        OpenCascadeSimplifiedModel,
                        ModelMetadata { vertices, triangles, edges },
                    ));
                } else {
                    // Fallback to original mesh if simplification fails
                    let vertices = get_vertex_count(&step_asset.mesh);
                    let triangles = get_triangle_count(&step_asset.mesh);
                    let edges = triangles * 3;
                    
                    commands.spawn((
                        Mesh3d(meshes.add(step_asset.mesh.clone())),
                        MeshMaterial3d(occt_simplified_material.clone()),
                        Transform::from_xyz(occt_simplified_pos.x, occt_simplified_pos.y, occt_simplified_pos.z)
                            .with_scale(Vec3::splat(scale_factor)),
                        GlobalTransform::default(),
                        RotatingModel,
                        OpenCascadeSimplifiedModel,
                        ModelMetadata { vertices, triangles, edges },
                    ));
                }
            }
            #[cfg(not(all(feature = "opencascade", feature = "meshopt")))]
            {
                let vertices = get_vertex_count(&step_asset.mesh);
                let triangles = get_triangle_count(&step_asset.mesh);
                let edges = triangles * 3; // Approximation
                
                commands.spawn((
                    Mesh3d(meshes.add(step_asset.mesh.clone())),
                    MeshMaterial3d(occt_simplified_material.clone()),
                    Transform::from_xyz(occt_simplified_pos.x, occt_simplified_pos.y, occt_simplified_pos.z)
                        .with_scale(Vec3::splat(scale_factor)),
                    GlobalTransform::default(),
                    RotatingModel,
                    OpenCascadeSimplifiedModel,
                    ModelMetadata { vertices, triangles, edges },
                ));
            }

            // Remove the resource to prevent re-execution
            commands.remove_resource::<StepModel>();
        }
    }
}

#[derive(Component)]
struct RotatingModel;

#[derive(Component)]
struct FoxtrotModel;

#[derive(Component)]
struct OpenCascadeModel;

#[derive(Component)]
struct FoxtrotSimplifiedModel;

#[derive(Component)]
struct OpenCascadeSimplifiedModel;

#[derive(Component)]
struct ModelMetadata {
    vertices: usize,
    triangles: usize,
    edges: usize,
}

fn rotate_models(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RotatingModel>>,
) {
    for mut transform in &mut query {
        transform.rotate_x(time.delta_secs() * 0.8);
    }
}

fn update_statistics(
    mut text_param_set: ParamSet<(
        Query<&mut Text, With<FoxtrotStatsDisplay>>,
        Query<&mut Text, With<OpenCascadeStatsDisplay>>,
        Query<&mut Text, With<FoxtrotSimplifiedStatsDisplay>>,
        Query<&mut Text, With<OpenCascadeSimplifiedStatsDisplay>>,
    )>,
    mut metadata_param_set: ParamSet<(
        Query<&ModelMetadata, (With<FoxtrotModel>, Without<OpenCascadeModel>, Without<FoxtrotSimplifiedModel>, Without<OpenCascadeSimplifiedModel>)>,
        Query<&ModelMetadata, (With<OpenCascadeModel>, Without<FoxtrotModel>, Without<FoxtrotSimplifiedModel>, Without<OpenCascadeSimplifiedModel>)>,
        Query<&ModelMetadata, (With<FoxtrotSimplifiedModel>, Without<FoxtrotModel>, Without<OpenCascadeModel>, Without<OpenCascadeSimplifiedModel>)>,
        Query<&ModelMetadata, (With<OpenCascadeSimplifiedModel>, Without<FoxtrotModel>, Without<OpenCascadeModel>, Without<FoxtrotSimplifiedModel>)>
    )>
) {
    // Update Foxtrot stats
    if let Ok(mut text) = text_param_set.p0().single_mut() {
        if let Ok(metadata) = metadata_param_set.p0().single() {
            **text = format!(
                "verts: {}\ntris: {}\nedges: {}\nbytes: {}",
                metadata.vertices, metadata.triangles, metadata.edges, metadata.vertices * 3 * 4  // Approximation: 3 floats per vertex, 4 bytes per float
            );
        }
    }

    // Update OpenCASCADE stats
    if let Ok(mut text) = text_param_set.p1().single_mut() {
        if let Ok(metadata) = metadata_param_set.p1().single() {
            **text = format!(
                "verts: {}\ntris: {}\nedges: {}\nbytes: {}",
                metadata.vertices, metadata.triangles, metadata.edges, metadata.vertices * 3 * 4  // Approximation: 3 floats per vertex, 4 bytes per float
            );
        }
    }

    // Update Foxtrot + Simplified stats
    if let Ok(mut text) = text_param_set.p2().single_mut() {
        if let Ok(metadata) = metadata_param_set.p2().single() {
            **text = format!(
                "verts: {}\ntris: {}\nedges: {}\nbytes: {}\nw/meshopt",
                metadata.vertices, metadata.triangles, metadata.edges, metadata.vertices * 3 * 4  // Approximation: 3 floats per vertex, 4 bytes per float
            );
        }
    }

    // Update OpenCASCADE + Simplified stats
    if let Ok(mut text) = text_param_set.p3().single_mut() {
        if let Ok(metadata) = metadata_param_set.p3().single() {
            **text = format!(
                "verts: {}\ntris: {}\nedges: {}\nbytes: {}\nw/meshopt",
                metadata.vertices, metadata.triangles, metadata.edges, metadata.vertices * 3 * 4  // Approximation: 3 floats per vertex, 4 bytes per float
            );
        }
    }
}

fn get_vertex_count(mesh: &Mesh) -> usize {
    if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy_mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
            _ => 0,
        }
    } else {
        0
    }
}

fn get_triangle_count(mesh: &Mesh) -> usize {
    if let Some(indices) = mesh.indices() {
        match indices {
            bevy_mesh::Indices::U32(indices) => indices.len() / 3,
            bevy_mesh::Indices::U16(indices) => indices.len() / 3,
        }
    } else {
        0
    }
}

fn calculate_and_print_distances(camera_pos: Vec3, model_positions: Res<ModelPositions>) {
    for (i, model_pos) in model_positions.positions.iter().enumerate() {
        let distance = camera_pos.distance(*model_pos);
        let model_name = match i {
            0 => "Foxtrot",
            1 => "OpenCASCADE", 
            2 => "Foxtrot Simplified",
            3 => "OpenCASCADE Simplified",
            _ => "Other Model"
        };
        
        println!("Distance to {}: {:.2}", model_name, distance);
    }
}

fn camera_control_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut camera_state: ResMut<CameraState>,
    model_positions: Res<ModelPositions>,
    mut query: Query<(&mut Projection, &mut Transform), With<Camera3d>>,
    windows: Query<&Window>,
) {
    let _window = windows.single();
    
    // Store old values to detect changes
    let old_zoom = camera_state.zoom;
    let old_translation = camera_state.translation;
    
    // Handle mouse wheel zoom first
    for event in mouse_wheel_events.read() {
        let zoom_delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 0.1,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y * 0.001,
        };
        
        // Update zoom level (with reasonable limits) - inverted so scroll forward = zoom in
        camera_state.zoom = (camera_state.zoom * (1.0 + zoom_delta)).clamp(0.1, 20.0);
    }
    
    // Handle middle mouse button translation
    // Use cursor events to update camera panning
    for cursor_event in cursor_moved_events.read() {
        if mouse_button_input.pressed(MouseButton::Middle) {
            let current_pos = Vec2::new(cursor_event.position.x, cursor_event.position.y);
            
            if let Some(start_pos) = camera_state.pan_start_pos {
                // Calculate the difference in screen space
                let delta = current_pos - start_pos;
                
                // Convert screen delta to world space delta
                // The conversion factor depends on the current zoom level and orthographic scale
                let scale_factor = 4.5 / camera_state.zoom;  // Current orthographic scale adjusted for panning sensitivity
                
                // Calculate world space translation (inverted because moving mouse right should move scene left)
                let world_delta = Vec2::new(delta.x * scale_factor * 0.001, -delta.y * scale_factor * 0.001);
                
                // Update camera translation
                camera_state.translation += world_delta;
            }
            
            // Update the start position for next frame
            camera_state.pan_start_pos = Some(current_pos);
        } else if !mouse_button_input.just_released(MouseButton::Middle) {
            // If middle button is not pressed but wasn't just released, reset the start position
            camera_state.pan_start_pos = None;
        }
    }
    
    // If middle button was just released, reset the start position
    if mouse_button_input.just_released(MouseButton::Middle) {
        camera_state.pan_start_pos = None;
    }
    
    // Update the camera's orthographic projection based on zoom level
    if let Ok((mut projection, mut transform)) = query.single_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            // Adjust the scale of the orthographic projection based on zoom
            ortho.scale = 4.5 / camera_state.zoom; // Invert zoom so that scroll in = zoom in
            
            // Apply translation based on camera state
            transform.translation.x = camera_state.translation.x;
            transform.translation.y = camera_state.translation.y;
        }
    }
    
    // Check if zoom or translation changed (after they have been processed)
    let zoom_changed = (camera_state.zoom - old_zoom).abs() > 0.001;
    let translation_changed = camera_state.translation != old_translation;
    
    if zoom_changed || translation_changed {
        // The camera position after transformations
        if let Ok((_, transform)) = query.single() {
            if zoom_changed {
                // Print zoom level instead of position when only zooming
                println!("Zoom level: {:.2}x (Camera z={})", camera_state.zoom, 1000.0); // The z-position is always 1000.0
            } else if translation_changed {
                // Print camera position only when translation changes (panning)
                println!("Camera position: x={}, y={}, z={}", 
                    transform.translation.x, 
                    transform.translation.y, 
                    transform.translation.z
                );
                
                // Calculate and print distances to each model
                calculate_and_print_distances(transform.translation, model_positions);
            }
        }
    }
}
