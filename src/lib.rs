use std::{path::PathBuf, env, fs::File};
use anyhow::Result;
pub use bary_macros::bary_app;
pub use bary_server::*;
pub use tar;
pub fn load(config_path: impl Into<PathBuf>) -> Result<()> {
    let path: PathBuf = config_path.into();
    println!("cargo:rerun-if-changed={}", path.display());
    let config = load_config(path.clone())?;
    let frontend_path = std::fs::canonicalize(config.frontend)?;
    let frontend_path = frontend_path.join("dist/");
    println!("cargo:rerun-if-changed={}", frontend_path.display());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::copy(path.clone(), out_path.join("bary.yaml"))?;
    let file = File::create(out_path.join("frontend.tar"))?;
    let mut archive = tar::Builder::new(file);
    archive.append_dir_all(".", frontend_path)?;
    archive.finish()?;
    Ok(())
}

#[macro_export]
macro_rules! frontend_setup {
    () => {
        {
            use std::io::Cursor;
            use $crate::tar::Archive;
            let frontend: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/frontend.tar"));
            let mut cursor = Cursor::new(frontend.to_vec());
            let mut archive = Archive::new(cursor);
            archive
        }
    };
}