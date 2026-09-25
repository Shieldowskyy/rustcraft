// build.rs – Compiles Cg shaders → GXP for the PS Vita platform.
// Automatically downloads psp2cgc if it is not installed.

use std::path::PathBuf;
use std::env;
use std::process::Command;

fn main() {
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // For non-Vita platforms – empty placeholders (GXM is not used)
    if target != "vita" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(out_dir.join("voxel.vert.gxp"), b"").unwrap();
        std::fs::write(out_dir.join("voxel.frag.gxp"), b"").unwrap();
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
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
