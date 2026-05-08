//! Integration tests for dynamic plugin loading
//!
//! These tests require a compiled plugin .so file.
//! Run `cargo build --release` in synapsis-plugins-example/hello_plugin first.

use synapsis_core::domain::{DynamicPluginLoader, PluginRegistry};

#[test]
fn test_load_hello_plugin() {
    // Path to the compiled hello_plugin.so
    let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../synapsis-plugins-example/hello_plugin/target/release/libhello_plugin.so");

    // Skip test if plugin doesn't exist (not built yet)
    if !plugin_path.exists() {
        println!("Skipping test: hello_plugin.so not found. Run 'cargo build --release' in synapsis-plugins-example/hello_plugin");
        return;
    }

    let mut loader = DynamicPluginLoader::new();
    let result = loader.load_plugin(&plugin_path);

    assert!(
        result.is_ok(),
        "Failed to load hello_plugin: {:?}",
        result.err()
    );

    let plugin = result.unwrap();
    let info = plugin.info();

    assert_eq!(info.id, "hello-plugin");
    assert_eq!(info.name, "Hello World Plugin");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn test_load_and_register_hello_plugin() {
    let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../synapsis-plugins-example/hello_plugin/target/release/libhello_plugin.so");

    if !plugin_path.exists() {
        println!("Skipping test: hello_plugin.so not found");
        return;
    }

    let mut loader = DynamicPluginLoader::new();
    let mut registry = PluginRegistry::new();

    let result = loader.load_and_register(&plugin_path, &mut registry);

    assert!(
        result.is_ok(),
        "Failed to load and register: {:?}",
        result.err()
    );
    assert_eq!(loader.loaded_count(), 1);
}

#[test]
fn test_unload_plugins() {
    let plugin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../synapsis-plugins-example/hello_plugin/target/release/libhello_plugin.so");

    if !plugin_path.exists() {
        println!("Skipping test: hello_plugin.so not found");
        return;
    }

    let mut loader = DynamicPluginLoader::new();
    let _ = loader.load_plugin(&plugin_path);

    assert_eq!(loader.loaded_count(), 1);

    let unload_result = loader.unload_all();
    assert!(unload_result.is_ok());
    assert_eq!(loader.loaded_count(), 0);
}
