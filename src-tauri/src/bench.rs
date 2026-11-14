//! Quick benchmarks: disk throughput (unbuffered) and a CPU stability stress test.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

static CANCEL: AtomicBool = AtomicBool::new(false);

pub fn cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    phase: String,
    percent: f64,
}

fn emit(app: &AppHandle, phase: &str, percent: f64) {
    let _ = app.emit("bench-progress", Progress { phase: phase.to_string(), percent });
}

pub fn list_drives() -> Vec<String> {
    ('C'..='Z')
        .filter(|c| Path::new(&format!("{c}:\\")).exists())
        .map(|c| format!("{c}:"))
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskResult {
    pub drive: String,
    pub size_mb: u32,
    pub seq_write_mbs: f64,
    pub seq_read_mbs: f64,
    pub rand_read_iops: f64,
    pub rand_read_mbs: f64,
}

/// Sector-aligned buffer (required by FILE_FLAG_NO_BUFFERING).
struct Aligned {
    raw: Vec<u8>,
    off: usize,
    len: usize,
}

impl Aligned {
    fn new(len: usize) -> Self {
        let raw = vec![0u8; len + 4096];
        let off = raw.as_ptr().align_offset(4096);
        Aligned { raw, off, len }
    }
    fn slice(&mut self) -> &mut [u8] {
        &mut self.raw[self.off..self.off + self.len]
    }
}

fn open(path: &Path, write: bool) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    if write {
        options.write(true).create(true).truncate(true);
    } else {
        options.read(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const NO_BUFFERING: u32 = 0x2000_0000;
        const WRITE_THROUGH: u32 = 0x8000_0000;
        options.custom_flags(NO_BUFFERING | if write { WRITE_THROUGH } else { 0 });
    }
    options.open(path)
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

pub fn disk_bench(app: &AppHandle, drive: &str, size_mb: u32) -> Result<DiskResult, String> {
    let letter = drive.trim().trim_end_matches([':', '\\']).to_uppercase();
    if letter.len() != 1 || !letter.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("invalid drive".into());
    }
    if !Path::new(&format!("{letter}:\\")).exists() {
        return Err("drive was not found".into());
    }
    let size_mb = size_mb.clamp(128, 2048);
    let dir = PathBuf::from(format!("{letter}:\\SetupHubBench"));
    fs::create_dir_all(&dir).map_err(|e| format!("cannot write to {letter}: {e}"))?;
    let path = dir.join("bench.tmp");

    CANCEL.store(false, Ordering::SeqCst);
    let result = run_disk(app, &path, &letter, size_mb);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(&dir);
    result
}

fn run_disk(app: &AppHandle, path: &Path, letter: &str, size_mb: u32) -> Result<DiskResult, String> {
    const BLOCK: usize = 1024 * 1024;
    let blocks = size_mb as usize;
    let mut buffer = Aligned::new(BLOCK);
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for chunk in buffer.slice().chunks_mut(8) {
        let value = xorshift(&mut seed).to_le_bytes();
        chunk.copy_from_slice(&value[..chunk.len()]);
    }

    // Sequential write
    emit(app, "Последовательная запись", 2.0);
    let started = Instant::now();
    {
        let mut file = open(path, true).map_err(|e| format!("cannot create test file: {e}"))?;
        for i in 0..blocks {
            if CANCEL.load(Ordering::SeqCst) {
                return Err("Остановлено".into());
            }
            file.write_all(buffer.slice()).map_err(|e| format!("write failed (disk full?): {e}"))?;
            if i % 16 == 0 {
                emit(app, "Последовательная запись", 2.0 + 33.0 * i as f64 / blocks as f64);
            }
        }
        file.sync_all().map_err(|e| e.to_string())?;
    }
    let seq_write = size_mb as f64 / started.elapsed().as_secs_f64().max(0.001);

    // Sequential read
    emit(app, "Последовательное чтение", 36.0);
    let started = Instant::now();
    {
        let mut file = open(path, false).map_err(|e| e.to_string())?;
        for i in 0..blocks {
            if CANCEL.load(Ordering::SeqCst) {
                return Err("Остановлено".into());
            }
            file.read_exact(buffer.slice()).map_err(|e| e.to_string())?;
            if i % 16 == 0 {
                emit(app, "Последовательное чтение", 36.0 + 34.0 * i as f64 / blocks as f64);
            }
        }
    }
    let seq_read = size_mb as f64 / started.elapsed().as_secs_f64().max(0.001);

    // Random 4K reads (queue depth 1)
    emit(app, "Случайное чтение 4K", 72.0);
    let mut small = Aligned::new(4096);
    let slots = (size_mb as u64 * 1024 * 1024) / 4096;
    let mut file = open(path, false).map_err(|e| e.to_string())?;
    let started = Instant::now();
    let mut ops = 0u32;
    while ops < 4000 && started.elapsed() < Duration::from_secs(5) {
        if CANCEL.load(Ordering::SeqCst) {
            return Err("Остановлено".into());
        }
        let position = (xorshift(&mut seed) % slots) * 4096;
        file.seek(SeekFrom::Start(position)).map_err(|e| e.to_string())?;
        file.read_exact(small.slice()).map_err(|e| e.to_string())?;
        ops += 1;
        if ops % 100 == 0 {
            emit(app, "Случайное чтение 4K", 72.0 + 26.0 * (ops as f64 / 4000.0).min(1.0));
        }
    }
    let secs = started.elapsed().as_secs_f64().max(0.001);
    let iops = ops as f64 / secs;

    emit(app, "Готово", 100.0);
    let round = |v: f64| (v * 10.0).round() / 10.0;
    crate::diag::log("INFO", &format!("disk bench {letter}: write {seq_write:.0} read {seq_read:.0} iops {iops:.0}"));
    Ok(DiskResult {
        drive: format!("{letter}:"),
        size_mb,
        seq_write_mbs: round(seq_write),
        seq_read_mbs: round(seq_read),
        rand_read_iops: iops.round(),
        rand_read_mbs: round(iops * 4096.0 / 1_048_576.0),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StressResult {
    pub threads: usize,
    pub seconds: u32,
    /// Millions of work units per second over the whole run.
    pub score: f64,
    /// How much throughput dropped between the first and the last quarter, in percent.
    pub throttle_percent: f64,
}

pub fn cpu_stress(app: &AppHandle, seconds: u32) -> Result<StressResult, String> {
    let seconds = seconds.clamp(10, 120);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    CANCEL.store(false, Ordering::SeqCst);

    let stop = Arc::new(AtomicBool::new(false));
    let counter = Arc::new(AtomicU64::new(0));
    let workers: Vec<_> = (0..threads)
        .map(|_| {
            let stop = stop.clone();
            let counter = counter.clone();
            std::thread::spawn(move || {
                let mut x = 1.0f64;
                while !stop.load(Ordering::Relaxed) {
                    for i in 0..20_000u32 {
                        x = (x * 1.000_000_1 + i as f64).sqrt().sin().abs() + 1.0;
                    }
                    std::hint::black_box(x);
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    let started = Instant::now();
    let mut samples: Vec<f64> = Vec::new();
    let mut last = 0u64;
    let mut cancelled = false;
    while started.elapsed() < Duration::from_secs(seconds as u64) {
        if CANCEL.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
        let now = counter.load(Ordering::Relaxed);
        samples.push((now - last) as f64 * 2.0);
        last = now;
        emit(app, "Нагрузка на процессор", 100.0 * started.elapsed().as_secs_f64() / seconds as f64);
    }
    stop.store(true, Ordering::SeqCst);
    for worker in workers {
        let _ = worker.join();
    }
    if cancelled {
        return Err("Остановлено".into());
    }

    let total = counter.load(Ordering::Relaxed) as f64;
    let quarter = (samples.len() / 4).max(1);
    let avg = |slice: &[f64]| slice.iter().sum::<f64>() / slice.len().max(1) as f64;
    let first = avg(&samples[..quarter.min(samples.len())]);
    let end = avg(&samples[samples.len().saturating_sub(quarter)..]);
    let throttle = if first > 0.0 { ((first - end) / first * 100.0).max(0.0) } else { 0.0 };

    emit(app, "Готово", 100.0);
    Ok(StressResult {
        threads,
        seconds,
        score: (total * 20_000.0 / seconds as f64 / 1_000_000.0 * 10.0).round() / 10.0,
        throttle_percent: (throttle * 10.0).round() / 10.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligned_buffer_is_aligned() {
        let mut buffer = Aligned::new(4096);
        assert_eq!(buffer.slice().as_ptr() as usize % 4096, 0);
        assert_eq!(buffer.slice().len(), 4096);
    }

    #[test]
    fn xorshift_changes_state() {
        let mut state = 1u64;
        let a = xorshift(&mut state);
        let b = xorshift(&mut state);
        assert_ne!(a, b);
    }
}
