use image::imageops::FilterType;
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ICON: &str = "packaging/app-icon.png";

    println!("cargo:rerun-if-changed={ICON}");
    if env::var("CARGO_CFG_TARGET_OS")? == "linux" {
        let icon = image::open(ICON)?;
        let output = PathBuf::from(env::var("OUT_DIR")?);
        for size in [16, 32, 64] {
            let pixels = icon
                .resize_exact(size, size, FilterType::Lanczos3)
                .into_rgba8();
            fs::write(output.join(format!("icon-{size}.rgba")), pixels.as_raw())?;
        }
    }
    Ok(())
}
