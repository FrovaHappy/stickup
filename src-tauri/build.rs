fn main() {
    std::env::remove_var("STATIC_VCRUNTIME");
    let dist = std::path::Path::new("../build");
    if !dist.exists() {
        let _ = std::fs::create_dir_all(dist);
    }

    fix_xwin_case_sensitivity();

    tauri_build::build()
}

fn fix_xwin_case_sensitivity() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" {
        return;
    }

    let libs_to_fix = [
        ("directml.lib", "DirectML.lib"),
        ("pathcch.lib", "PathCch.lib"),
    ];

    let mut search_dirs = Vec::new();

    // 1. Inspect RUSTFLAGS and target-specific rustflags
    for env_var in [
        "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS",
        "CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS",
        "CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUSTFLAGS",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
    ] {
        if let Ok(flags) = std::env::var(env_var) {
            for part in flags.split(|c| c == ' ' || c == '\x1f') {
                if let Some(path_str) = part
                    .strip_prefix("-Lnative=")
                    .or_else(|| part.strip_prefix("-L"))
                {
                    let p = std::path::PathBuf::from(path_str.trim_start_matches("native="));
                    if p.is_dir() {
                        search_dirs.push(p);
                    }
                }
            }
        }
    }

    // 2. Inspect XWIN_CACHE_DIR and common workspace / cache locations
    let mut candidate_roots = Vec::new();
    if let Ok(xwin_cache) = std::env::var("XWIN_CACHE_DIR") {
        candidate_roots.push(std::path::PathBuf::from(xwin_cache));
    }
    candidate_roots.push(std::path::PathBuf::from("target/.xwin-cache"));
    candidate_roots.push(std::path::PathBuf::from("../target/.xwin-cache"));
    candidate_roots.push(std::path::PathBuf::from(".xwin-cache"));
    if let Ok(home) = std::env::var("HOME") {
        candidate_roots.push(std::path::PathBuf::from(&home).join(".cache/xwin"));
        candidate_roots.push(std::path::PathBuf::from(&home).join(".xwin"));
    }

    for root in candidate_roots {
        if root.is_dir() {
            let sdk_um = root.join("xwin/sdk/lib/um");
            if sdk_um.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&sdk_um) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            search_dirs.push(entry.path());
                        }
                    }
                }
            }
        }
    }

    let out_dir = std::env::var("OUT_DIR").map(std::path::PathBuf::from).ok();
    let compat_dir = out_dir.as_ref().map(|d| d.join("xwin_compat_libs"));
    if let Some(ref dir) = compat_dir {
        let _ = std::fs::create_dir_all(dir);
    }

    let mut found_any = false;
    for dir in &search_dirs {
        for (lower, target_name) in &libs_to_fix {
            let lower_path = dir.join(lower);
            let target_path = dir.join(target_name);

            if lower_path.is_file() {
                if !target_path.exists() {
                    create_symlink_or_copy(lower, &lower_path, &target_path);
                }

                if let Some(ref cdir) = compat_dir {
                    let fallback_path = cdir.join(target_name);
                    if !fallback_path.exists() {
                        create_symlink_or_copy(lower, &lower_path, &fallback_path);
                        found_any = true;
                    }
                }
            }
        }
    }

    if found_any {
        if let Some(ref cdir) = compat_dir {
            println!("cargo:rustc-link-search=native={}", cdir.display());
        }
    }
}

fn create_symlink_or_copy(filename: &str, src_full: &std::path::Path, dst: &std::path::Path) {
    #[cfg(unix)]
    {
        let symlink_src = if dst.parent() == src_full.parent() {
            std::path::Path::new(filename)
        } else {
            src_full
        };

        if std::os::unix::fs::symlink(symlink_src, dst).is_ok() {
            return;
        }
    }
    let _ = filename; // silence unused warning on windows
    let _ = std::fs::copy(src_full, dst);
}

