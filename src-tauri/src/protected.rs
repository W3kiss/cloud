use std::io::Read;

// The key is split into four independent fragments. Reconstructing it at
// runtime prevents the compiler from keeping a single readable key table next
// to the masked resource. This is obfuscation, not a replacement for real
// secret management: it raises the cost of casual string extraction.
const KEY_A: [u8; 8] = [0x5A, 0xC3, 0x19, 0xE7, 0x42, 0x8B, 0xF0, 0x2D];
const KEY_B: [u8; 8] = [0x77, 0xA1, 0x3C, 0xD6, 0x64, 0x0E, 0x9B, 0x58];
const KEY_C: [u8; 8] = [0x31, 0xAF, 0x7C, 0xE2, 0x0D, 0x46, 0xB9, 0x95];
const KEY_D: [u8; 8] = [0x6A, 0x28, 0xD4, 0x83, 0x1E, 0xFA, 0x50, 0xBF];

fn key_at(index: usize) -> u8 {
    let slot = (index.wrapping_mul(7) + 13) % 32;
    match slot {
        0..=7 => KEY_A[slot],
        8..=15 => KEY_B[slot - 8],
        16..=23 => KEY_C[slot - 16],
        _ => KEY_D[slot - 24],
    }
}

fn unmask(masked: &[u8]) -> Vec<u8> {
    masked
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let mixed = *byte ^ key_at(index) ^ ((index as u8).wrapping_mul(31));
            let rotation = ((index % 7) + 1) as u32;
            mixed.rotate_right(rotation)
        })
        .collect()
}

/// Decompress the protected resource that was produced by `build.rs`.
pub fn decode_catalog(masked: &[u8]) -> Result<String, String> {
    let compressed = unmask(masked);
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut raw = String::new();
    decoder
        .read_to_string(&mut raw)
        .map_err(|error| format!("catalog resource is damaged: {error}"))?;
    Ok(raw)
}
