// build.rs – Kompiluje shadery Cg → GXP dla platformy PS Vita
// Automatycznie pobiera psp2cgc jeśli nie jest zainstalowany.

use std::path::PathBuf;
use std::env;
use std::process::Command;

fn main() {
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Dla platform innych niż Vita – puste placeholdery (GXM nie jest używany)
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
    // 1. Znajdź lub zainstaluj psp2cgc
    // ----------------------------------------------------------------
    let psp2cgc = find_or_install_psp2cgc(&vitasdk);

    match psp2cgc {
        Some(ref compiler) => {
            println!("cargo:warning=Znaleziono psp2cgc: {}", compiler);
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
            println!("cargo:warning=✓ Shadery skompilowane pomyślnie!");
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
                println!("cargo:warning=✓ Shadery GXP skopiowane z vita/shaders/");
            } else if vert_pb.exists() && frag_pb.exists() {
                std::fs::copy(&vert_pb, out_dir.join("voxel.vert.gxp")).unwrap();
                std::fs::copy(&frag_pb, out_dir.join("voxel.frag.gxp")).unwrap();
                println!("cargo:warning=✓ Pre-skompilowane shadery załadowane z vita/shaders/prebuilt/");
            } else {
                eprintln!("\n================================================================");
                eprintln!("BŁĄD BUDOWANIA: Nie można znaleźć kompilatora shaderów psp2cgc!");
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
// Wyszukuje psp2cgc w znanych lokalizacjach, a jeśli nie znajdzie
// próbuje go pobrać automatycznie z GitHub releases.
// ----------------------------------------------------------------
fn find_or_install_psp2cgc(vitasdk: &str) -> Option<String> {
    // 1. Sprawdź znane lokalizacje
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

    // 2. Nie znaleziono – spróbuj pobrać z GitHub
    println!("cargo:warning=psp2cgc nie znaleziony. Próbuję pobrać automatycznie...");

    let install_dir = format!("{}/.local/share/psp2cgc", env::var("HOME").unwrap_or_default());
    let install_path = format!("{}/psp2cgc", install_dir);

    if try_download_psp2cgc(&install_dir, &install_path) {
        println!("cargo:warning=✓ psp2cgc pobrano i zainstalowano do: {}", install_path);
        return Some(install_path);
    }

    None
}

fn try_download_psp2cgc(install_dir: &str, install_path: &str) -> bool {
    // Utwórz katalog instalacyjny
    if std::fs::create_dir_all(install_dir).is_err() {
        return false;
    }

    // URL do psp2cgc z bekwunu/psp2cgc-oss (open-source reimplementation)
    // Alternatywnie z archiwów vita-toolchain
    let urls = [
        // psp2cgc Linux x86_64 prebuilt z vita-headers/psp2cgc-oss
        "https://github.com/SonicMastr/psp2cgc-oss/releases/latest/download/psp2cgc-linux",
        // Backup: z vita-toolchain releases
        "https://github.com/vitasdk/vdpm/releases/download/2022/psp2cgc-linux-x86_64",
    ];

    for url in &urls {
        println!("cargo:warning=Pobieranie z: {}", url);

        // Spróbuj wget
        let ok = Command::new("wget")
            .args(["-q", "-O", install_path, url])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !ok {
            // Spróbuj curl
            let ok2 = Command::new("curl")
                .args(["-sL", "-o", install_path, url])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok2 {
                continue;
            }
        }

        // Sprawdź czy plik jest sensowny (> 10KB = prawdopodobnie binarny)
        if let Ok(meta) = std::fs::metadata(install_path) {
            if meta.len() > 10_000 {
                // Nadaj prawa wykonania
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
                // Plik zbyt mały – prawdopodobnie 404 lub redirect HTML
                let _ = std::fs::remove_file(install_path);
            }
        }
    }

    false
}

fn compile_shader(psp2cgc: &str, input: PathBuf, output: PathBuf, profile: &str) {
    println!("cargo:warning=Kompilowanie shadera: {:?}", input.file_name().unwrap());

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
            panic!("Kompilacja shadera {:?} nieudana! Kod: {}", input, s);
        }
        Err(e) => {
            panic!("Nie można uruchomić psp2cgc: {}", e);
        }
    }
}

// ================================================================
// Minimalne placeholder GXP – poprawny nagłówek, ale brak kodu
// shadera. Powoduje błąd GXM przy init, ale pozwala na kompilację.
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
