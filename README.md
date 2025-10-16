# bevy_step_loader

A Bevy plugin that adds support for loading STEP and STP files as Bevy Mesh assets.

## Features

- Load STEP files directly into Bevy scenes as meshes
- Support for multiple triangulation backends
- Optional mesh optimisation using meshopt
- Asynchronous asset loading
- Mesh simplification for performance optimisation

## Triangulation Backends

This crate supports two different triangulation engines with different trade-offs:

### Foxtrot (Default)
- **Pros**: Pure Rust implementation, faster STEP parsing and triangulation, smaller binary size
- **Cons**: Less robust triangulation, particularly with complex geometries and NURBS surfaces
- **wasm**: Naturally this is easier to use in `wasm` builds

### OpenCascade (OCCT) - Optional Feature
- **Pros**: More robust triangulation, better handling of complex geometries and NURBS, well-established tooling
- **Cons**: C++ wrapper dependency, slower triangulation, larger binary size

## Prerequisites

For the OpenCascade backend, you need a C++ library to link into, so install some `libstdc++`:

```sh
sudo apt update
sudo apt install libstdc++-12-dev
```

Or check your distribution's package manager for equivalent packages.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_step_loader = { git = "https://github.com/alphastrata/bevy_step_loader" }
```

## Usage

### Basic Usage (with default Foxtrot backend)

```rust
use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(asset_server: Res<AssetServer>) {
    // Load a STEP file (using a file present in your assets directory)
    let step_handle: Handle<StepAsset> = asset_server.load("22604_bcab4db9_0001_2.step");
    
    // The STEP file will be automatically triangulated and available as a mesh
    println!("Loading STEP file...");
}
```

### Using OpenCascade Backend

To use the OpenCascade backend for better triangulation:

```toml
[dependencies]
bevy_step_loader = { git = "https://github.com/alphastrata/bevy_step_loader", features = ["opencascade"] }
```

### Enabling Mesh Optimisation

To enable vertex cache optimisation for better rendering performance:

```toml
[dependencies]
bevy_step_loader = { git = "https://github.com/alphastrata/bevy_step_loader", features = ["meshopt"] }
```

### Enabling Mesh Simplification

To enable mesh simplification for performance optimisation:

```toml
[dependencies]
bevy_step_loader = { git = "https://github.com/alphastrata/bevy_step_loader", features = ["meshopt"] }
```

## Supported File Extensions

- `.step`
- `.stp`
- `.STEP`
- `.STP`

## Examples

The repository includes several examples:

- `basic_load.rs` - Basic STEP file loading
- `headless_test.rs` - Headless testing of STEP loading
- `usage.rs` - Comprehensive example showing all configurations with visualisation
- `step_3d_scene.rs` - 3D scene example based on the default Bevy 3D scene that loads STEP files from the asset server

## Features

- `opencascade`: Enable OpenCascade backend for more robust triangulation
- `meshopt`: Enable mesh optimisation and simplification using meshopt crate

## API Reference

### StepAsset

The main asset type that represents a loaded STEP file.

#### Methods

- `simplify_mesh(ratio: f32, error_threshold: f32) -> Result<(), StepLoaderError>`: 
  Simplifies the mesh using meshopt decimation algorithm.
  - `ratio`: Target reduction ratio (0.0 to 1.0, where 1.0 means no reduction and 0.5 means 50% reduction)
  - `error_threshold`: Maximum allowed error for the simplification
  - Only available when the `meshopt` feature is enabled

## License
MIT