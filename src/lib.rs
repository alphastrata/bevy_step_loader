use bevy_app::{Plugin, App};
use bevy_asset::{Asset, AssetLoader, LoadContext, io::Reader, RenderAssetUsages, AssetApp};
use bevy_reflect::TypePath;
use bevy_mesh::{Mesh, Indices};
use wgpu_types::PrimitiveTopology;

#[cfg(feature = "meshopt")]
use meshopt;
#[cfg(feature = "meshopt")]
use bytemuck;



#[derive(Debug)]
pub enum StepLoaderError {
    IoError(std::io::Error),
    #[cfg(feature = "opencascade")]
    OcctError(String),
    FoxtrotError(String),
    ParseError(String),
}

impl std::fmt::Display for StepLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepLoaderError::IoError(e) => write!(f, "IO error: {}", e),
            #[cfg(feature = "opencascade")]
            StepLoaderError::OcctError(e) => write!(f, "OpenCASCADE error: {}", e),
            StepLoaderError::FoxtrotError(e) => write!(f, "Foxtrot triangulation error: {}", e),
            StepLoaderError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for StepLoaderError {}

impl From<std::io::Error> for StepLoaderError {
    fn from(error: std::io::Error) -> Self {
        StepLoaderError::IoError(error)
    }
}

#[cfg(feature = "opencascade")]
impl From<String> for StepLoaderError {
    fn from(error: String) -> Self {
        StepLoaderError::OcctError(error)
    }
}

pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StepAsset>()
            .register_asset_loader(StepLoader);
    }
}

// The asset representing a STEP file
#[derive(Asset, TypePath, Debug, Clone)]
pub struct StepAsset {
    pub mesh: Mesh,
}

impl StepAsset {
    /// Simplify the mesh using meshopt decimation
    /// 
    /// # Arguments
    /// * `ratio` - Target reduction ratio (0.0 to 1.0, where 1.0 means no reduction and 0.5 means 50% reduction)
    /// * `error_threshold` - Maximum allowed error for the simplification
    /// 
    /// # Returns
    /// * `Ok(())` if simplification was successful
    /// * `Err(StepLoaderError)` if simplification failed or meshopt feature is not enabled
    #[cfg(feature = "meshopt")]
    pub fn simplify_mesh(&mut self, ratio: f32, error_threshold: f32) -> Result<(), StepLoaderError> {
        use std::mem;

        // Extract vertex positions
        let positions = match self.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(positions) => match positions {
                bevy_mesh::VertexAttributeValues::Float32x3(pos) => pos.clone(),
                _ => return Err(StepLoaderError::ParseError("Expected Float32x3 positions".to_string())),
            },
            None => return Err(StepLoaderError::ParseError("No position attribute found".to_string())),
        };

        // Extract indices
        let original_indices: Vec<u32> = match self.mesh.indices() {
            Some(indices) => match indices {
                Indices::U32(indices) => indices.clone(),
                Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
            },
            None => return Err(StepLoaderError::ParseError("No indices found".to_string())),
        };

        // Prepare data for meshopt
        let vertices: Vec<f32> = positions
            .iter()
            .flat_map(|&[x, y, z]| [x, y, z])
            .collect();

        let target_index_count = (original_indices.len() as f32 * ratio) as usize;
        let target_error = error_threshold;

        // Create vertex adapter
        let vertex_size = 3 * mem::size_of::<f32>();
        let vertex_adapter = match meshopt::VertexDataAdapter::new(
            bytemuck::cast_slice(&vertices),
            vertex_size,
            0,
        ) {
            Ok(adapter) => adapter,
            Err(_) => return Err(StepLoaderError::ParseError("Failed to create vertex adapter".to_string())),
        };

        // Perform simplification
        let mut error_result: f32 = 0.0;
        let simplified_indices = meshopt::simplify(
            &original_indices,
            &vertex_adapter,
            target_index_count,
            target_error,
            meshopt::SimplifyOptions::LockBorder,
            Some(&mut error_result),
        );

        // Update the mesh with simplified indices (mutable borrow only when needed)
        if let Some(indices) = self.mesh.indices_mut() {
            *indices = Indices::U32(simplified_indices.clone());
        }

        println!("Mesh simplified: {} -> {} indices (error: {})", original_indices.len(), simplified_indices.len(), error_result);
        
        Ok(())
    }

    /// Simplify the mesh using meshopt decimation
    /// 
    /// This method is only available when the `meshopt` feature is enabled.
    /// If the feature is not enabled, this method will always return an error.
    #[cfg(not(feature = "meshopt"))]
    pub fn simplify_mesh(&mut self, _ratio: f32, _error_threshold: f32) -> Result<(), StepLoaderError> {
        Err(StepLoaderError::ParseError("Mesh simplification requires the 'meshopt' feature to be enabled".to_string()))
    }
}

// The loader for STEP files
#[derive(Default)]
pub struct StepLoader;

impl AssetLoader for StepLoader {
    type Asset = StepAsset;
    type Settings = ();
    type Error = StepLoaderError;

    fn extensions(&self) -> &[&str] {
        &["step", "stp", "STEP", "STP"]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mesh = triangulate_step_file(&bytes)?;

        Ok(StepAsset { mesh })
    }
}

/// Triangulate the STEP file data into a Bevy Mesh.
/// Depending on the feature flag, it uses either OpenCASCADE (opencascade) or Foxtrot library.
///
/// The 'opencascade' feature, means you'll build it via the wrapper, some cmake etc deps and fanalging may be required
/// however, it is SIGNIFICANTLY more robust and can handle a wider variety of STEP files, and their miscellaneous shitfuckery.
fn triangulate_step_file(step_data: &[u8]) -> Result<Mesh, StepLoaderError> {
    #[cfg(feature = "opencascade")]
    {
        triangulate_with_occt(step_data)
    }
    #[cfg(not(feature = "opencascade"))]
    {
        triangulate_with_foxtrot(step_data)
    }
}

#[cfg(feature = "opencascade")]
fn triangulate_with_occt(step_data: &[u8]) -> Result<Mesh, StepLoaderError> {
    use opencascade::primitives::Shape;
    use opencascade::mesh::Mesher;

    let temp_path = std::env::temp_dir().join("temp_step_file.step");
    std::fs::write(&temp_path, step_data)?;

    let shape_to_mesh = Shape::read_step(temp_path.to_str().unwrap())
        .map_err(|e| StepLoaderError::OcctError(format!("OCCT failed to read STEP file: {:?}", e)))?;

    let occt_mesh = Mesher::new(&shape_to_mesh).mesh();

    let vertices: Vec<[f32; 3]> = occt_mesh
        .vertices
        .iter()
        .map(|v| [v.x as f32, v.y as f32, v.z as f32])
        .collect();

    let indices: Vec<u32> = occt_mesh.indices.iter().map(|&i| i as u32).collect();

    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(), // Using the asset API directly
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    bevy_mesh.insert_indices(Indices::U32(indices));
    
    // Compute normals for proper lighting
    bevy_mesh.compute_normals();

    #[cfg(feature = "meshopt")]
    {
        optimise_mesh(&mut bevy_mesh)?;
    }

    Ok(bevy_mesh)
}

#[allow(dead_code)]
fn triangulate_with_foxtrot(step_data: &[u8]) -> Result<Mesh, StepLoaderError> {
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate4 as triangulate;

    let flat = StepFile::strip_flatten(step_data);
    let step = StepFile::parse(&flat);
    let (triangulated_mesh, _stats) = triangulate(&step);

    let vertices: Vec<[f32; 3]> = triangulated_mesh
        .verts
        .iter()
        .map(|v| [v.pos.x as f32, v.pos.y as f32, v.pos.z as f32])
        .collect();

    let indices: Vec<u32> = triangulated_mesh
        .triangles
        .iter()
        .flat_map(|t| [t.verts.x, t.verts.y, t.verts.z])
        .collect();

    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(), // Using the asset API directly
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    bevy_mesh.insert_indices(Indices::U32(indices));
    
    // Compute normals for proper lighting
    bevy_mesh.compute_normals();

    #[cfg(feature = "meshopt")]
    {
        optimise_mesh(&mut bevy_mesh)?;
    }

    Ok(bevy_mesh)
}

#[cfg(feature = "meshopt")]
fn optimise_mesh(mesh: &mut Mesh) -> Result<(), StepLoaderError> {
    let positions: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(positions) => match positions {
            bevy_mesh::VertexAttributeValues::Float32x3(pos) => pos.to_vec(),
            _ => return Err(StepLoaderError::ParseError("Expected Float32x3 positions".to_string())),
        },
        None => return Err(StepLoaderError::ParseError("No position attribute found".to_string())),
    };

    let mut indices: Vec<u32> = match mesh.indices() {
        Some(indices) => match indices {
            Indices::U32(idx) => idx.to_vec(),
            Indices::U16(idx) => idx.iter().map(|&i| i as u32).collect(),
        },
        None => return Err(StepLoaderError::ParseError("No indices found".to_string())),
    };

    if !indices.is_empty() && !positions.is_empty() {
        meshopt::optimise_vertex_cache_in_place(&mut indices, positions.len());
        
        *mesh.indices_mut().unwrap() = Indices::U32(indices);
    }

    Ok(())
}
