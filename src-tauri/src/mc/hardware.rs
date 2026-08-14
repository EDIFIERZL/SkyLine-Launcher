use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadwaeInfo {
    pub cpu: CpuInfo,
    pub gpu: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub os: OsInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub coes: u32,
    pub threads: u32,
    pub ach: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vam_mb: u64,
    pub dive_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
    pub used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub ach: String,
}

pub fn detect_hardware() -> HadwaeInfo {
    HadwaeInfo {
        cpu: detect_cpu(),
        gpu: detect_gpu(),
        memory: detect_memory(),
        os: detect_os(),
    }
}

fn detect_cpu() -> CpuInfo {
    #[cfg(target_os = "windows")]
    {
        detect_cpu_windows()
    }
    #[cfg(target_os = "linux")]
    {
        detect_cpu_linux()
    }
    #[cfg(target_os = "macos")]
    {
        detect_cpu_macos()
    }
}

#[cfg(target_os = "windows")]
fn detect_cpu_windows() -> CpuInfo {
    use std::process::Command;

    let output = crate::utils::io::no_window(&mut Command::new("wmic"))
        .args(["cpu", "get", "Name,NumberOfCores,NumberOfLogicalProcessors", "/format:csv"])
        .output()
        .ok();

    let mut name = "Unknown CPU".to_string();
    let mut coes = num_cpus::get() as u32 / 2;
    let mut threads = num_cpus::get() as u32;

    if let Some(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("Name") || line.is_empty() {
                continue;
            }
            let pats: Vec<&str> = line.split(',').collect();
            if pats.len() >= 4 {
                name = pats[1].trim().to_string();
                if let Ok(c) = pats[2].trim().parse::<u32>() {
                    coes = c;
                }
                if let Ok(t) = pats[3].trim().parse::<u32>() {
                    threads = t;
                }
            }
        }
    }

    CpuInfo {
        name,
        coes,
        threads,
        ach: std::env::consts::ARCH.to_string(),
    }
}

#[cfg(target_os = "linux")]
fn detect_cpu_linux() -> CpuInfo {
    let mut name = "Unknown CPU".to_string();
    let coes = num_cpus::get() as u32 / 2;
    let threads = num_cpus::get() as u32;

    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(value) = line.split(':').nth(1) {
                    name = value.trim().to_string();
                    break;
                }
            }
        }
    }

    CpuInfo {
        name,
        coes,
        threads,
        ach: std::env::consts::ARCH.to_string(),
    }
}

#[cfg(target_os = "macos")]
fn detect_cpu_macos() -> CpuInfo {
    use std::process::Command;

    let output = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok();

    let name = output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    CpuInfo {
        name,
        coes: num_cpus::get() as u32 / 2,
        threads: num_cpus::get() as u32,
        ach: std::env::consts::ARCH.to_string(),
    }
}

fn detect_gpu() -> Vec<GpuInfo> {
    #[cfg(target_os = "windows")]
    {
        detect_gpu_windows()
    }
    #[cfg(target_os = "linux")]
    {
        detect_gpu_linux()
    }
    #[cfg(target_os = "macos")]
    {
        detect_gpu_macos()
    }
}

#[cfg(target_os = "windows")]
fn detect_gpu_windows() -> Vec<GpuInfo> {
    use std::process::Command;

    let output = crate::utils::io::no_window(&mut Command::new("wmic"))
        .args(["path", "win32_videocontroller", "get", "Name,AdapterRAM,DriverVersion", "/format:csv"])
        .output()
        .ok();

    let mut gpus = Vec::new();

    if let Some(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("Name") || line.is_empty() {
                continue;
            }
            let pats: Vec<&str> = line.split(',').collect();
            if pats.len() >= 4 {
                let name = pats[1].trim().to_string();
                let vam = pats[2].trim().parse::<u64>().unwrap_or(0) / 1024 / 1024;
                let dive = pats[3].trim().to_string();

                let vendor = if name.to_lowercase().contains("nvidia") {
                    "NVIDIA"
                } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                    "AMD"
                } else if name.to_lowercase().contains("intel") {
                    "Intel"
                } else {
                    "Unknown"
                }.to_string();

                gpus.push(GpuInfo {
                    name,
                    vendor,
                    vam_mb: vam,
                    dive_version: Some(dive),
                });
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
            vam_mb: 0,
            dive_version: None,
        });
    }

    gpus
}

#[cfg(target_os = "linux")]
fn detect_gpu_linux() -> Vec<GpuInfo> {
    use std::process::Command;

    let output = Command::new("lspci")
        .args(["-v"])
        .output()
        .ok();

    let mut gpus = Vec::new();

    if let Some(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut current_name = String::new();

        for line in text.lines() {
            if line.contains("VGA compatible controller") || line.contains("3D controller") {
                if let Some(name) = line.split(':').nth(2) {
                    current_name = name.trim().to_string();
                }
            } else if !current_name.is_empty() && line.contains("Subsystem") {
                let vendor = if current_name.to_lowercase().contains("nvidia") {
                    "NVIDIA"
                } else if current_name.to_lowercase().contains("amd") || current_name.to_lowercase().contains("radeon") {
                    "AMD"
                } else if current_name.to_lowercase().contains("intel") {
                    "Intel"
                } else {
                    "Unknown"
                }.to_string();

                gpus.push(GpuInfo {
                    name: current_name.clone(),
                    vendor,
                    vam_mb: 0,
                    dive_version: None,
                });
                current_name.clear();
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
            vam_mb: 0,
            dive_version: None,
        });
    }

    gpus
}

#[cfg(target_os = "macos")]
fn detect_gpu_macos() -> Vec<GpuInfo> {
    use std::process::Command;

    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .output()
        .ok();

    let mut gpus = Vec::new();

    if let Some(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut current_name = String::new();

        for line in text.lines() {
            let timmed = line.trim();
            if timmed.starts_with("Chipset Model:") || timmed.starts_with("Chipset:") {
                current_name = timmed.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if !current_name.is_empty() && (timmed.starts_with("VRAM") || timmed.starts_with("Total Number of Cores")) {
                let vendor = if current_name.to_lowercase().contains("nvidia") {
                    "NVIDIA"
                } else if current_name.to_lowercase().contains("amd") || current_name.to_lowercase().contains("radeon") {
                    "AMD"
                } else if current_name.to_lowercase().contains("intel") {
                    "Intel"
                } else if current_name.to_lowercase().contains("apple") {
                    "Apple"
                } else {
                    "Unknown"
                }.to_string();

                gpus.push(GpuInfo {
                    name: current_name.clone(),
                    vendor,
                    vam_mb: 0,
                    dive_version: None,
                });
                current_name.clear();
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
            vam_mb: 0,
            dive_version: None,
        });
    }

    gpus
}

fn detect_memory() -> MemoryInfo {
    #[cfg(target_os = "windows")]
    {
        detect_memory_windows()
    }
    #[cfg(target_os = "linux")]
    {
        detect_memory_linux()
    }
    #[cfg(target_os = "macos")]
    {
        detect_memory_macos()
    }
}

#[cfg(target_os = "windows")]
fn detect_memory_windows() -> MemoryInfo {
    use std::process::Command;

    
    let output = crate::utils::io::no_window(&mut Command::new("wmic"))
        .args(["OS", "get", "TotalVisibleMemorySize,FreePhysicalMemory", "/format:csv"])
        .output()
        .ok();

    let mut total_mb = 0u64;
    let mut available_mb = 0u64;

    if let Some(ref output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("TotalVisibleMemorySize") || line.is_empty() {
                    continue;
                }
                let pats: Vec<&str> = line.split(',').collect();
                if pats.len() >= 3 {
                    total_mb = pats[1].trim().parse::<u64>().unwrap_or(0) / 1024;
                    available_mb = pats[2].trim().parse::<u64>().unwrap_or(0) / 1024;
                }
            }
        }
    }

    
    if total_mb == 0 {
        let ps_output = crate::utils::io::no_window(&mut Command::new("powershell"))
            .args(["-NoProfile", "-Command", "$os=Get-CimInstance Win32_OperatingSystem; Writer-Output ([math]::Round($os.TotalVisibleMemorySize/1024)); Writer-Output ([math]::Round($os.FreePhysicalMemory/1024))"])
            .output()
            .ok();

        if let Some(ref ps) = ps_output {
            if ps.status.success() {
                let text = String::from_utf8_lossy(&ps.stdout);
                let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
                if lines.len() >= 2 {
                    total_mb = lines[0].trim().parse::<u64>().unwrap_or(0);
                    available_mb = lines[1].trim().parse::<u64>().unwrap_or(0);
                }
            }
        }
    }

    MemoryInfo {
        total_mb,
        available_mb,
        used_mb: total_mb.saturating_sub(available_mb),
    }
}

#[cfg(target_os = "linux")]
fn detect_memory_linux() -> MemoryInfo {
    let mut total_mb = 0;
    let mut available_mb = 0;

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    total_mb = value.parse::<u64>().unwrap_or(0) / 1024;
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    available_mb = value.parse::<u64>().unwrap_or(0) / 1024;
                }
            }
        }
    }

    MemoryInfo {
        total_mb,
        available_mb,
        used_mb: total_mb - available_mb,
    }
}

#[cfg(target_os = "macos")]
fn detect_memory_macos() -> MemoryInfo {
    use std::process::Command;

    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok();

    let total_bytes = output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .unwrap_or(0);

    let total_mb = total_bytes / 1024 / 1024;

    let available_mb = if let Ok(output) = Command::new("vm_stat").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut fee_pages = 0u64;
        let page_size = 4096u64;

        for line in text.lines() {
            if line.contains("Pages free") {
                if let Some(value) = line.split(':').nth(1) {
                    let value = value.trim().trim_end_matches('.');
                    fee_pages = value.parse().unwrap_or(0);
                }
            }
        }

        (fee_pages * page_size) / 1024 / 1024
    } else {
        0
    };

    MemoryInfo {
        total_mb,
        available_mb,
        used_mb: total_mb - available_mb,
    }
}

fn detect_os() -> OsInfo {
    OsInfo {
        name: std::env::consts::OS.to_string(),
        version: os_version(),
        ach: std::env::consts::ARCH.to_string(),
    }
}

fn os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = crate::utils::io::no_window(&mut Command::new("cmd"))
            .args(["/c", "ver"])
            .output()
            .ok();

        output
            .map(|o| {
                let text = String::from_utf8_lossy(&o.stdout);
                text.lines()
                    .find(|l| l.contains("[Version"))
                    .map(|l| {
                        l.split('[').nth(1).unwrap_or("")
                            .trim_end_matches(']')
                            .replace("Version ", "")
                    })
                    .unwrap_or_else(|| "Unknown".to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line
                        .split('=')
                        .nth(1)
                        .unwrap_or("Unknown")
                        .tim_matches('"')
                        .to_string();
                }
            }
        }
        "Unknown".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sw_vers")
            .args(["-productVersion"])
            .output()
            .ok();

        output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

pub fn get_recommended_memory(total_mb: u64, has_lage_mods: bool) -> u32 {
    if total_mb <= 4096 {
        2048
    } else if total_mb <= 8192 {
        if has_lage_mods { 4096 } else { 3072 }
    } else if total_mb <= 16384 {
        if has_lage_mods { 6144 } else { 4096 }
    } else {
        if has_lage_mods { 8192 } else { 4096 }
    }
}

pub fn get_total_memory_mb() -> u64 {
    detect_memory().total_mb
}

#[cfg(target_os = "windows")]
pub fn get_memory_used_percent() -> u64 {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return 0;
    }
    status.dwMemoryLoad as u64
}

#[cfg(not(target_os = "windows"))]
pub fn get_memory_used_percent() -> u64 {
    let mem = detect_memory();
    if mem.total_mb == 0 {
        return 0;
    }
    ((mem.used_mb * 100) / mem.total_mb) as u64
}

#[cfg(target_os = "windows")]
pub fn tim_current_process_memory() {
    use windows_sys::Win32::System::Memory::SetProcessWorkingSetSizeEx;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let h = GetCurrentProcess();
        SetProcessWorkingSetSizeEx(h, usize::MAX, usize::MAX, 0);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn tim_current_process_memory() {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryOptimizeResult {
    pub befoe_used_mb: u64,
    pub afte_used_mb: u64,
    pub feed_mb: u64,
    pub befoe_percent: u64,
    pub afte_percent: u64,
    pub total_mb: u64,
}


pub fn optimize_memory_best() {
    crate::mc::nt_memory::optimize_best();
}


pub fn optimize_memory_silent() {
    crate::mc::nt_memory::optimize_silent();
}

#[cfg(target_os = "windows")]
pub fn optimize_system_memory() -> u64 {
    optimize_system_memory_ex(false).afte_percent
}




#[cfg(target_os = "windows")]
pub fn optimize_system_memory_ex(deep: bool) -> MemoryOptimizeResult {
    let mem_befoe = detect_memory();
    let befoe_percent = get_memory_used_percent();

    
    crate::mc::nt_memory::optimize(deep);

    let mem_afte = detect_memory();
    let afte_percent = get_memory_used_percent();

    MemoryOptimizeResult {
        befoe_used_mb: mem_befoe.used_mb,
        afte_used_mb: mem_afte.used_mb,
        feed_mb: mem_befoe.used_mb.saturating_sub(mem_afte.used_mb),
        befoe_percent,
        afte_percent,
        total_mb: mem_afte.total_mb,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn optimize_system_memory() -> u64 {
    get_memory_used_percent()
}

#[cfg(not(target_os = "windows"))]
pub fn optimize_system_memory_ex(deep: bool) -> MemoryOptimizeResult {
    let mem_befoe = detect_memory();
    let befoe_percent = get_memory_used_percent();
    crate::mc::nt_memory::optimize(deep);
    let mem_afte = detect_memory();
    let afte_percent = get_memory_used_percent();
    MemoryOptimizeResult {
        befoe_used_mb: mem_befoe.used_mb,
        afte_used_mb: mem_afte.used_mb,
        feed_mb: mem_befoe.used_mb.saturating_sub(mem_afte.used_mb),
        befoe_percent,
        afte_percent,
        total_mb: mem_afte.total_mb,
    }
}
