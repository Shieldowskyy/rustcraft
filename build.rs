// build.rs – Compiles Cg shaders → GXP for the PS Vita platform.
// Automatically downloads psp2cgc if it is not installed.

use std::path::PathBuf;
use std::env;
use std::process::Command;

fn main() {
    emit_build_info();

    let target = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    emit_embedded_assets(&out_dir, &target);

    // For non-Vita platforms – empty placeholders (GXM is not used)
    if target != "vita" {
        std::fs::write(out_dir.join("voxel.vert.gxp"), b"").unwrap();
        std::fs::write(out_dir.join("voxel.frag.gxp"), b"").unwrap();
        return;
    }

    let vitasdk = env::var("VITASDK").unwrap_or_else(|_| "/usr/local/vitasdk".into());
    let shader_dir = PathBuf::from("vita/shaders");

    // ----------------------------------------------------------------
    // 1. Locate or install psp2cgc
    // ----------------------------------------------------------------
    let psp2cgc = find_or_install_psp2cgc(&vitasdk);

    match psp2cgc {
        Some(ref compiler) => {
            println!("cargo:warning=Found psp2cgc: {}", compiler);
            compile_shader(
                compiler,
                shader_dir.join("voxel.vert.cg"),
                out_dir.join("voxel.vert.gxp"),
                "sce_vp_psp2",
            );
            compile_shader(
                compiler,
                shader_dir.join("voxel.frag.cg"),
                out_dir.join("voxel.frag.gxp"),
                "sce_fp_psp2",
            );
            println!("cargo:warning=✓ Shaders compiled successfully!");
        }
        None => {
            let vert_src = shader_dir.join("voxel.vert.gxp");
            let frag_src = shader_dir.join("voxel.frag.gxp");
            let prebuilt = PathBuf::from("vita/shaders/prebuilt");
            let vert_pb = prebuilt.join("voxel.vert.gxp");
            let frag_pb = prebuilt.join("voxel.frag.gxp");

            if vert_src.exists() && frag_src.exists() {
                std::fs::copy(&vert_src, out_dir.join("voxel.vert.gxp")).unwrap();
                std::fs::copy(&frag_src, out_dir.join("voxel.frag.gxp")).unwrap();
                println!("cargo:warning=✓ GXP shaders copied from vita/shaders/");
            } else if vert_pb.exists() && frag_pb.exists() {
                std::fs::copy(&vert_pb, out_dir.join("voxel.vert.gxp")).unwrap();
                std::fs::copy(&frag_pb, out_dir.join("voxel.frag.gxp")).unwrap();
                println!("cargo:warning=✓ Pre-compiled shaders loaded from vita/shaders/prebuilt/");
            } else {
                eprintln!("\n================================================================");
                eprintln!("BUILD ERROR: Unable to find psp2cgc shader compiler!");
                eprintln!("================================================================\n");
                std::fs::write(out_dir.join("voxel.vert.gxp"), MINIMAL_VERT_GXP).unwrap();
                std::fs::write(out_dir.join("voxel.frag.gxp"), MINIMAL_FRAG_GXP).unwrap();
            }
        }
    }

    println!("cargo:rerun-if-changed=vita/shaders/voxel.vert.cg");
    println!("cargo:rerun-if-changed=vita/shaders/voxel.frag.cg");
    println!("cargo:rerun-if-changed=vita/shaders/prebuilt/");
}

fn emit_embedded_assets(out_dir: &std::path::Path, target_os: &str) {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let single_executable_target = matches!(target_os, "linux" | "windows" | "macos" | "freebsd")
        || env::var_os("CARGO_FEATURE_XBOX360").is_some();
    let mut assets = Vec::new();

    if single_executable_target && target_arch != "wasm32" {
        let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let soundtracks_dir = project_dir.join("soundtracks");
        collect_soundtrack_assets(
            &soundtracks_dir,
            &soundtracks_dir,
            &mut assets,
        );
    }

    assets.sort_by(|a, b| a.0.cmp(&b.0));
    let mut generated = String::from("pub static SOUNDTRACKS: &[(&str, &[u8])] = &[\n");
    for (relative_path, absolute_path) in assets {
        generated.push_str(&format!(
            "    ({:?}, include_bytes!({:?})),\n",
            relative_path,
            absolute_path.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    std::fs::write(out_dir.join("embedded_assets.rs"), generated).unwrap();
}

fn collect_soundtrack_assets(root: &std::path::Path, dir: &std::path::Path, assets: &mut Vec<(String, PathBuf)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_soundtrack_assets(root, &path, assets);
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();
        if matches!(extension.to_ascii_lowercase().as_str(), "ogg" | "opus" | "wav") {
            let relative = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            assets.push((format!("soundtracks/{relative}"), path));
        }
    }
}

// ----------------------------------------------------------------
// Searches for psp2cgc in known directories, and if not found,
// attempts to automatically download it from GitHub releases.
// ----------------------------------------------------------------
fn find_or_install_psp2cgc(vitasdk: &str) -> Option<String> {
    // 1. Check known locations
    let candidates = [
        format!("{}/bin/psp2cgc", vitasdk),
        format!("{}/arm-vita-eabi/bin/psp2cgc", vitasdk),
        format!("{}/.local/share/psp2cgc/psp2cgc", env::var("HOME").unwrap_or_default()),
        format!("{}/.cargo/bin/psp2cgc", env::var("HOME").unwrap_or_default()),
        "/usr/local/bin/psp2cgc".into(),
        "/usr/bin/psp2cgc".into(),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }

    // 2. Not found – try downloading from GitHub
    println!("cargo:warning=psp2cgc not found. Attempting automatic download...");

    let install_dir = format!("{}/.local/share/psp2cgc", env::var("HOME").unwrap_or_default());
    let install_path = format!("{}/psp2cgc", install_dir);

    if try_download_psp2cgc(&install_dir, &install_path) {
        println!("cargo:warning=✓ psp2cgc downloaded and installed to: {}", install_path);
        return Some(install_path);
    }

    None
}

fn try_download_psp2cgc(install_dir: &str, install_path: &str) -> bool {
    // Create installation directory
    if std::fs::create_dir_all(install_dir).is_err() {
        return false;
    }

    // URLs for psp2cgc from SonicMastr/psp2cgc-oss or vita-toolchain
    let urls = [
        "https://github.com/SonicMastr/psp2cgc-oss/releases/latest/download/psp2cgc-linux",
        "https://github.com/vitasdk/vdpm/releases/download/2022/psp2cgc-linux-x86_64",
    ];

    for url in &urls {
        println!("cargo:warning=Downloading from: {}", url);

        // Try wget
        let ok = Command::new("wget")
            .args(["-q", "-O", install_path, url])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !ok {
            // Try curl
            let ok2 = Command::new("curl")
                .args(["-sL", "-o", install_path, url])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok2 {
                continue;
            }
        }

        // Check if file is valid (> 10KB = likely binary)
        if let Ok(meta) = std::fs::metadata(install_path) {
            if meta.len() > 10_000 {
                // Set executable permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        install_path,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
                return true;
            } else {
                // File too small – probably 404 or HTML redirect
                let _ = std::fs::remove_file(install_path);
            }
        }
    }

    false
}

fn compile_shader(psp2cgc: &str, input: PathBuf, output: PathBuf, profile: &str) {
    println!("cargo:warning=Compiling shader: {:?}", input.file_name().unwrap());

    let status = Command::new(psp2cgc)
        .args([
            "-profile", profile,
            "-o", output.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=  ✓ {:?} → {:?}", input.file_name().unwrap(), output.file_name().unwrap());
        }
        Ok(s) => {
            panic!("Shader compilation failed for {:?}! Code: {}", input, s);
        }
        Err(e) => {
            panic!("Cannot execute psp2cgc: {}", e);
        }
    }
}

// ================================================================
// Minimal placeholder GXP – valid header, empty shader bytecode.
// ================================================================
const MINIMAL_VERT_GXP: &[u8] = &[
    0x47, 0x58, 0x50, 0x00, // magic "GXP\0"
    0x01, 0x00, 0x00, 0x00, // version = 1
    0x00, 0x00, 0x00, 0x00, // type = vertex
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

const MINIMAL_FRAG_GXP: &[u8] = &[
    0x47, 0x58, 0x50, 0x00, // magic "GXP\0"
    0x01, 0x00, 0x00, 0x00, // version = 1
    0x01, 0x00, 0x00, 0x00, // type = fragment
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

fn emit_build_info() {
    let build_datetime = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();
            let days = secs / 86400;
            let day_secs = secs % 86400;
            let hours = day_secs / 3600;
            let minutes = (day_secs % 3600) / 60;
            let seconds = day_secs % 60;
            format!("Unix+{}d {:02}:{:02}:{:02}", days, hours, minutes, seconds)
        });

    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".to_string());
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "3".to_string());

    let platform_name = match target_os.as_str() {
        "vita" => "PS Vita",
        "android" => "Android",
        "xbox360" | "xenon" => "Xbox 360",
        "windows" => "Windows",
        "linux" => "Linux",
        "macos" => "macOS",
        "ios" => "iOS",
        _ => &target_os,
    };

    let platform_full = format!("{} ({})", platform_name, target_arch);

    let mut features = Vec::new();
    for (k, _) in env::vars() {
        if let Some(feat) = k.strip_prefix("CARGO_FEATURE_") {
            features.push(feat.to_lowercase());
        }
    }
    features.sort();
    let features_str = if features.is_empty() {
        "none".to_string()
    } else {
        features.join(",")
    };

    let flags_str = format!("opt={}, feats=[{}]", opt_level, features_str);

    let full_info = format!(
        "RustCraft v{} [{}] | {} | {} | {}",
        env!("CARGO_PKG_VERSION"),
        profile,
        build_datetime,
        platform_full,
        flags_str
    );

    println!("cargo:rustc-env=RUSTCRAFT_BUILD_DATETIME={}", build_datetime);
    println!("cargo:rustc-env=RUSTCRAFT_BUILD_PROFILE={}", profile);
    println!("cargo:rustc-env=RUSTCRAFT_BUILD_TARGET={}", target);
    println!("cargo:rustc-env=RUSTCRAFT_BUILD_PLATFORM={}", platform_full);
    println!("cargo:rustc-env=RUSTCRAFT_BUILD_FLAGS={}", flags_str);
    println!("cargo:rustc-env=RUSTCRAFT_BUILD_INFO={}", full_info);
}
