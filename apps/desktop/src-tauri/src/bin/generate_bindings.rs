fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../packages/contracts/src/bindings.ts");
    if std::env::args().nth(1).as_deref() == Some("--check") {
        let candidate =
            std::env::temp_dir().join(format!("lifeos-contracts-{}.ts", std::process::id()));
        lifeos_desktop_lib::export_bindings(&candidate)?;
        let matches = std::fs::read(&candidate)? == std::fs::read(&destination)?;
        std::fs::remove_file(candidate)?;
        if !matches {
            return Err("generated TypeScript contracts are stale".into());
        }
    } else {
        lifeos_desktop_lib::export_bindings(destination)?;
    }
    Ok(())
}
