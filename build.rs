// build.rs – Kompiluje shadery Cg → GXP dla platformy PS Vita
// Wywoływane przez Cargo przed budowaniem projektu.

use std::path::PathBuf;
use std::env;
use std::process::Command;

fn main() {
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Kompilacja shaderów tylko dla PS Vita
    if target != "vita" {
        // Dla innych platform generujemy puste placeholdery (web/desktop nie używają GXP)
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        std::fs::write(out_dir.join("voxel.vert.gxp"), b"").unwrap();
        std::fs::write(out_dir.join("voxel.frag.gxp"), b"").unwrap();
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vitasdk = env::var("VITASDK").unwrap_or_else(|_| "/usr/local/vitasdk".into());
    let shader_dir = PathBuf::from("vita/shaders");

    // Szukamy psp2cgc w kilku lokalizacjach
    let psp2cgc_paths = [
        format!("{}/bin/psp2cgc", vitasdk),
        format!("{}/arm-vita-eabi/bin/psp2cgc", vitasdk),
        "/usr/bin/psp2cgc".into(),
        "/usr/local/bin/psp2cgc".into(),
    ];

    let psp2cgc = psp2cgc_paths.iter()
        .find(|p| std::path::Path::new(p).exists())
        .cloned();

    match psp2cgc {
        Some(compiler) => {
            println!("cargo:warning=Znaleziono psp2cgc: {}", compiler);
            compile_shader(
                &compiler,
                shader_dir.join("voxel.vert.cg"),
                out_dir.join("voxel.vert.gxp"),
                "sce_vp_psp2",
            );
            compile_shader(
                &compiler,
                shader_dir.join("voxel.frag.cg"),
                out_dir.join("voxel.frag.gxp"),
                "sce_fp_psp2",
            );
        }
        None => {
            // psp2cgc niedostępny – używamy pre-skompilowanych shaderów z vita/shaders/prebuilt/
            println!("cargo:warning=psp2cgc nie znaleziony. Używam pre-skompilowanych GXP z vita/shaders/prebuilt/");
            println!("cargo:warning=Aby zainstalować psp2cgc: vdpm install psp2cgc-linux");

            let prebuilt = PathBuf::from("vita/shaders/prebuilt");
            let vert_prebuilt = prebuilt.join("voxel.vert.gxp");
            let frag_prebuilt = prebuilt.join("voxel.frag.gxp");

            if vert_prebuilt.exists() && frag_prebuilt.exists() {
                std::fs::copy(&vert_prebuilt, out_dir.join("voxel.vert.gxp")).unwrap();
                std::fs::copy(&frag_prebuilt, out_dir.join("voxel.frag.gxp")).unwrap();
                println!("cargo:warning=Pre-skompilowane shadery załadowane pomyślnie.");
            } else {
                // Awaryjnie: puste bajty (build się skompiluje ale GXM zwróci błąd przy inicjalizacji)
                eprintln!("BŁĄD: Brak pre-skompilowanych GXP shaderów!");
                eprintln!("  Zainstaluj psp2cgc LUB umieść pliki .gxp w vita/shaders/prebuilt/");
                eprintln!("  Instrukcja: https://github.com/vitasdk/vdpm");
                std::fs::write(out_dir.join("voxel.vert.gxp"), MINIMAL_VERT_GXP).unwrap();
                std::fs::write(out_dir.join("voxel.frag.gxp"), MINIMAL_FRAG_GXP).unwrap();
                println!("cargo:warning=UWAGA: Użyto minimalnych placeholder GXP – shadery mogą nie działać!");
            }
        }
    }

    println!("cargo:rerun-if-changed=vita/shaders/voxel.vert.cg");
    println!("cargo:rerun-if-changed=vita/shaders/voxel.frag.cg");
    println!("cargo:rerun-if-changed=vita/shaders/prebuilt/");
}

fn compile_shader(psp2cgc: &str, input: PathBuf, output: PathBuf, profile: &str) {
    let status = Command::new(psp2cgc)
        .args([
            "-profile", profile,
            "-o", output.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .status()
        .expect("Nie można uruchomić psp2cgc");

    if !status.success() {
        panic!("Kompilacja shadera {:?} nieudana! Kod: {}", input, status);
    }
    println!("cargo:warning=Skompilowano shader: {:?} → {:?}", input, output);
}

// ============================================================
// Minimalne placeholder GXP – tylko poprawny header GXP.
// NIE renderują nic poprawnie, ale pozwalają na kompilację.
// Zastąp je prawdziwymi GXP z psp2cgc!
// Format GXP: magic(4) + version(4) + type(4) + ...
// ============================================================
const MINIMAL_VERT_GXP: &[u8] = &[
    // GXP magic "GXP\0" + version 1
    0x47, 0x58, 0x50, 0x00, // magic
    0x01, 0x00, 0x00, 0x00, // version = 1
    0x00, 0x00, 0x00, 0x00, // type = vertex
    // Reszta wymagana do minimalnego parsowania
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

const MINIMAL_FRAG_GXP: &[u8] = &[
    0x47, 0x58, 0x50, 0x00, // magic
    0x01, 0x00, 0x00, 0x00, // version = 1
    0x01, 0x00, 0x00, 0x00, // type = fragment
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];
