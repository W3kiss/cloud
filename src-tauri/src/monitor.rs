//! Live hardware stats (CPU / RAM / GPU / temperatures), emitted once per second.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

use crate::tweaks::run_powershell;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
struct Slow {
    gpu: Option<f64>,
    gpu_temp: Option<f64>,
    gpu_name: Option<String>,
    cpu_temp: Option<f64>,
}

static SLOW: Mutex<Slow> = Mutex::new(Slow {
    gpu: None,
    gpu_temp: None,
    gpu_name: None,
    cpu_temp: None,
});

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HwStats {
    cpu: f32,
    cpu_name: String,
    cpu_temp: Option<f64>,
    ram_used_gb: f64,
    ram_total_gb: f64,
    ram_percent: f64,
    gpu: Option<f64>,
    gpu_name: Option<String>,
    gpu_temp: Option<f64>,
}

/// GPU load/temperature and ACPI temperature come from slow sources (PowerShell,
/// nvidia-smi), so they are sampled on their own cadence.
const SLOW_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$gpu = $null
try { $s = (Get-Counter '\GPU Engine(*engtype_3D)\Utilization Percentage' -ErrorAction Stop).CounterSamples | Measure-Object CookedValue -Sum; $gpu = [math]::Min(100, [math]::Round($s.Sum, 0)) } catch {}
$nv = ''
try { $nv = [string](& nvidia-smi --query-gpu=name,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null | Select-Object -First 1) } catch {}
$t = $null
try { $z = Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop | Select-Object -First 1; if ($z) { $t = [math]::Round($z.CurrentTemperature / 10 - 273.15, 0) } } catch {}
ConvertTo-Json -Compress -InputObject @{ gpu = $gpu; nv = $nv; cpuTemp = $t }
"#;

fn sample_slow() {
    let Ok(report) = run_powershell(SLOW_SCRIPT) else { return };
    let (Some(a), Some(b)) = (report.find('{'), report.rfind('}')) else { return };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&report[a..=b]) else { return };

    let mut slow = Slow {
        gpu: value.get("gpu").and_then(|v| v.as_f64()),
        gpu_temp: None,
        gpu_name: None,
        cpu_temp: value.get("cpuTemp").and_then(|v| v.as_f64()).filter(|t| *t > 0.0 && *t < 120.0),
    };

    // "NVIDIA GeForce RTX 3060, 12, 55": name, load %, temperature °C
    if let Some(nv) = value.get("nv").and_then(|v| v.as_str()) {
        let parts: Vec<&str> = nv.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            slow.gpu_name = Some(parts[0].to_string());
            slow.gpu = parts[1].parse::<f64>().ok().or(slow.gpu);
            slow.gpu_temp = parts[2].parse::<f64>().ok();
        }
    }
    if let Ok(mut guard) = SLOW.lock() {
        *guard = slow;
    }
}

pub fn start(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    std::thread::spawn(|| {
        while RUNNING.load(Ordering::SeqCst) {
            sample_slow();
            for _ in 0..30 {
                if !RUNNING.load(Ordering::SeqCst) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    });

    std::thread::spawn(move || {
        let mut sys = System::new_all();
        let cpu_name = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_default();
        while RUNNING.load(Ordering::SeqCst) {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let total = sys.total_memory() as f64;
            let used = sys.used_memory() as f64;
            let slow = SLOW.lock().map(|s| s.clone()).ok();
            let stats = HwStats {
                cpu: sys.global_cpu_usage(),
                cpu_name: cpu_name.clone(),
                cpu_temp: slow.as_ref().and_then(|s| s.cpu_temp),
                ram_used_gb: (used / 1_073_741_824.0 * 10.0).round() / 10.0,
                ram_total_gb: (total / 1_073_741_824.0 * 10.0).round() / 10.0,
                ram_percent: if total > 0.0 { (used / total * 1000.0).round() / 10.0 } else { 0.0 },
                gpu: slow.as_ref().and_then(|s| s.gpu),
                gpu_name: slow.as_ref().and_then(|s| s.gpu_name.clone()),
                gpu_temp: slow.as_ref().and_then(|s| s.gpu_temp),
            };
            let _ = app.emit("hw-stats", stats);
            std::thread::sleep(Duration::from_millis(1000));
        }
    });
}

pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}
