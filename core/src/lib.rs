pub mod analyze;
pub mod data;
pub mod finder;
pub mod render;
pub mod seed;
pub mod world;

pub use analyze::{analyze, Analysis};
pub use world::World;

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WorldHandle {
        w: crate::World,
        analysis: String,
    }

    #[wasm_bindgen]
    impl WorldHandle {
        #[wasm_bindgen(constructor)]
        pub fn new(bytes: &[u8]) -> Result<WorldHandle, JsError> {
            let w = crate::World::parse(bytes).map_err(|e| JsError::new(&e))?;
            let a = crate::analyze(&w);
            let analysis = serde_json::to_string(&a).unwrap_or_default();
            Ok(WorldHandle { w, analysis })
        }
        pub fn width(&self) -> i32 {
            self.w.header.width
        }
        pub fn height(&self) -> i32 {
            self.w.header.height
        }
        pub fn analysis(&self) -> String {
            self.analysis.clone()
        }
        pub fn render(&self, layers: u32) -> Vec<u8> {
            crate::render::render(&self.w, layers)
        }
        pub fn describe(&self, x: i32, y: i32) -> String {
            crate::render::describe(&self.w, x, y)
        }
        pub fn hints(&self) -> String {
            let a: crate::Analysis = crate::analyze(&self.w);
            serde_json::to_string(&crate::finder::hints(&a)).unwrap_or_default()
        }
        /// Spoiler-free verdict for a goals JSON.
        pub fn judge(&self, goals_json: &str) -> String {
            let g: crate::finder::Goals = serde_json::from_str(goals_json).unwrap_or_default();
            let a = crate::analyze(&self.w);
            serde_json::to_string(&crate::finder::judge(&a, &g)).unwrap_or_default()
        }
    }

    /// Build a full Terraria seed string (the format the game's "copy seed" button gives).
    #[wasm_bindgen]
    pub fn build_seed(opts_json: &str) -> String {
        let o: crate::seed::SeedOptions = serde_json::from_str(opts_json).unwrap_or_default();
        crate::seed::build(&o)
    }

    #[wasm_bindgen]
    pub fn seed_catalog() -> String {
        crate::seed::catalog_json()
    }
}
