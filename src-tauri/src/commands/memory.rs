use crate::mc::hardware::{
    get_memory_used_percent, optimize_system_memory_ex,
    MemoryOptimizeResult,
};
use std::sync::atomic::{AtomicBool, Ordering};

static PERIODIC_RUNNING: AtomicBool = AtomicBool::new(false);


#[tauri::command]
pub fn start_periodic_optimization() {
    if PERIODIC_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        while PERIODIC_RUNNING.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_secs(120));
            if !PERIODIC_RUNNING.load(Ordering::SeqCst) {
                break;
            }
            optimize_system_memory_ex(false);
        }
    });
}


#[tauri::command]
pub fn stop_periodic_optimization() {
    PERIODIC_RUNNING.store(false, Ordering::SeqCst);
}



#[tauri::command]
pub fn optimize_memory_aggressive() -> MemoryOptimizeResult {
    optimize_system_memory_ex(true)
}


#[tauri::command]
pub async fn optimize_memory_silent() -> MemoryOptimizeResult {
    tokio::task::spawn_blocking(|| crate::mc::nt_memory::optimize_silent())
        .await
        .unwrap();
    optimize_system_memory_ex(false)
}


#[tauri::command]
pub async fn optimize_memory_best() -> MemoryOptimizeResult {
    tokio::task::spawn_blocking(|| crate::mc::nt_memory::optimize_best())
        .await
        .unwrap();
    optimize_system_memory_ex(false)
}


#[tauri::command]
pub fn get_memory_usage() -> u64 {
    get_memory_used_percent()
}
