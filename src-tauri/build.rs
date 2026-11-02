fn main() {
    std::env::remove_var("STATIC_VCRUNTIME");
    let dist = std::path::Path::new("../build");
    if !dist.exists() {
        let _ = std::fs::create_dir_all(dist);
    }
    tauri_build::build()
}
