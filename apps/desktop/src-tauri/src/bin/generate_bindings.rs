fn main() {
    let destination = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../packages/contracts/src/bindings.ts");
    lifeos_desktop_lib::export_bindings(destination).expect("export contracts");
}
