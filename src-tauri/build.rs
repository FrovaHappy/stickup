fn main() {
    let dist = std::path::Path::new("../build");
    if !dist.exists() {
        let _ = std::fs::create_dir_all(dist);
    }
    tauri_build::build()
}
