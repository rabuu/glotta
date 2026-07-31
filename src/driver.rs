use std::fs;
use std::path::PathBuf;

use tracing::info;

pub struct Driver {
    input_path: PathBuf,
    source: String,
}

impl Driver {
    pub fn new(input_path: PathBuf) -> Option<Self> {
        let source = fs::read_to_string(&input_path).ok()?;

        info!("read file '{}'", input_path.display());

        Some(Self { input_path, source })
    }
}
