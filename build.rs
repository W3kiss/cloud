use flate2::Compression;
use flate2::write::GzEncoder;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const MASK_KEY: [u8; 32] = [
    0x5A, 0xC3, 0x19, 0xE7, 0x42, 0x8B, 0xF0, 0x2D, 0x77, 0xA1, 0x3C, 0xD6, 0x64, 0x0E, 0x9B, 0x58,
    0x31, 0xAF, 0x7C, 0xE2, 0x0D, 0x46, 0xB9, 0x95, 0x6A, 0x28, 0xD4, 0x83, 0x1E, 0xFA, 0x50, 0xBF,
];

/// Compress and mask the catalog at compile time. The generated blob is the
/// only copy that gets linked into the release binary; `catalog.json` itself is
/// never embedded as a string literal.
fn mask(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(index, byte)| {
            let key = MASK_KEY[(index.wrapping_mul(7) + 13) % MASK_KEY.len()];
            let rotation = ((index % 7) + 1) as u32;
            byte.rotate_left(rotation) ^ key ^ ((index as u8).wrapping_mul(31))
        })
        .collect()
}

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-changed=catalog.json");

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by Cargo"),
    );
    let catalog_path = manifest_dir.join("catalog.json");
    let raw = fs::read(&catalog_path).expect("catalog.json should be readable");

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(&raw)
        .expect("catalog should compress without errors");
    let compressed = encoder
        .finish()
        .expect("catalog compression should finish without errors");

    let output_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set by Cargo"));
    fs::write(output_dir.join("catalog.masked"), mask(&compressed))
        .expect("masked catalog should be written to OUT_DIR");
}
