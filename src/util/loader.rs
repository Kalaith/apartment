/// Cross-platform file loading utilities for WASM and native
/// 
/// In WASM, files are embedded at compile time using include_str!
/// In native, files are loaded from disk with embedded fallback

/// Macro to load a JSON config file with WASM compatibility.
/// Usage: load_json_config!("assets/config.json", GameConfig)
/// 
/// This embeds the file at compile time for WASM and loads from disk for native.
#[macro_export]
macro_rules! load_json_config {
    ($path:literal, $type:ty) => {{
        #[cfg(target_arch = "wasm32")]
        let json: &str = include_str!(concat!("../../", $path));
        
        #[cfg(not(target_arch = "wasm32"))]
        let json: String = std::fs::read_to_string($path)
            .unwrap_or_else(|_| include_str!(concat!("../../", $path)).to_string());
        
        #[cfg(target_arch = "wasm32")]
        let result: Result<$type, _> = serde_json::from_str(json);
        
        #[cfg(not(target_arch = "wasm32"))]
        let result: Result<$type, _> = serde_json::from_str(&json);
        
        result
    }};
}

/// Load JSON with a default fallback if parsing fails
#[macro_export]
macro_rules! load_json_or_default {
    ($path:literal, $type:ty) => {{
        match $crate::load_json_config!($path, $type) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse {}: {}", $path, e);
                <$type>::default()
            }
        }
    }};
}
