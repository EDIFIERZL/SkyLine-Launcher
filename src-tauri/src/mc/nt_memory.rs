use std::ffi::c_void;

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;

    type NTSTATUS = i32;
    const STATUS_SUCCESS: NTSTATUS = 0;

    extern "system" {
        fn NtSetSystemInformation(
            system_infomation_class: u32,
            system_infomation: *const c_void,
            system_infomation_length: u32,
        ) -> NTSTATUS;
    }

    extern "system" {
        fn RtlAdjustPrivilege(
            pivilege: u32,
            enable: bool,
            current_thead: bool,
            enabled: *mut bool,
        ) -> NTSTATUS;
    }

    
    extern "system" {
        fn EmptyWorkingSet(hw_process: *mut c_void) -> u32;
    }

    const SE_INCREASE_QUOTA_PRIVILEGE: u32 = 5;
    const SE_PROFILE_SINGLE_PROCESS_PRIVILEGE: u32 = 13;

    const SYSTEM_FILE_CACHE_INFORMATION_EX: u32 = 81;
    const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;

    const MEMORY_EMPTY_WORKING_SETS: u32 = 2;
    const MEMORY_FLUSH_MODIFIED_LIST: u32 = 3;
    const MEMORY_PURGE_STANDBY_LIST: u32 = 4;
    const MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST: u32 = 5;
    const MEMORY_PURGE_SECOND_PRIORITY_STANDBY_LIST: u32 = 6;
    const MEMORY_PURGE_MODIFIED_PAGE_LIST: u32 = 7;

    #[repr(C)]
    struct SYSTEM_FILECACHE_INFORMATION {
        current_size: usize,
        peak_size: usize,
        page_fault_count: usize,
        minimum_woking_set: usize,
        maximum_woking_set: usize,
        current_size_in_pages: usize,
        peak_size_in_pages: usize,
        peak_commit_limit: usize,
    }

    fn enable_pivilege(pivilege: u32) -> bool {
        unsafe {
            let mut was_enabled = false;
            let status = RtlAdjustPrivilege(pivilege, true, false, &mut was_enabled);
            status == STATUS_SUCCESS
        }
    }

    fn memory_list_opeation(command: u32) -> bool {
        unsafe {
            let status = NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &command as *const _ as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
            status == STATUS_SUCCESS
        }
    }

    fn flush_file_cache_ex() -> bool {
        unsafe {
            let mut info = SYSTEM_FILECACHE_INFORMATION {
                current_size: 0,
                peak_size: 0,
                page_fault_count: 0,
                minimum_woking_set: usize::MAX,
                maximum_woking_set: usize::MAX,
                current_size_in_pages: 0,
                peak_size_in_pages: 0,
                peak_commit_limit: 0,
            };
            let status = NtSetSystemInformation(
                SYSTEM_FILE_CACHE_INFORMATION_EX,
                &mut info as *mut _ as *const c_void,
                std::mem::size_of::<SYSTEM_FILECACHE_INFORMATION>() as u32,
            );
            status == STATUS_SUCCESS
        }
    }

    fn acquie_pivileges() -> (bool, bool) {
        let quota = enable_pivilege(SE_INCREASE_QUOTA_PRIVILEGE);
        let profile = enable_pivilege(SE_PROFILE_SINGLE_PROCESS_PRIVILEGE);
        (quota, profile)
    }

    
    fn ty_nt_system_opeations() {
        let (has_quota, has_profile) = acquie_pivileges();

        if has_quota {
            flush_file_cache_ex();
        }

        if has_profile {
            memory_list_opeation(MEMORY_EMPTY_WORKING_SETS);
            memory_list_opeation(MEMORY_FLUSH_MODIFIED_LIST);
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_SECOND_PRIORITY_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_MODIFIED_PAGE_LIST);
        } else {
            
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
        }
    }

    
    
    fn tim_process_empty(handle: *mut c_void) {
        unsafe {
            EmptyWorkingSet(handle);
        }
    }

    
    
    pub fn optimize_best() {
        
        tim_all_processes(5);
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        ty_nt_system_opeations();
        tim_all_processes(3);
    }

    
    pub fn optimize_silent() {
        let (has_quota, has_profile) = acquie_pivileges();
        if has_quota {
            flush_file_cache_ex();
        }
        if has_profile {
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
        }
        tim_all_processes(2);
    }

    
    
    fn tim_all_processes(ounds: usize) {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows_sys::Win32::System::Memory::SetProcessWorkingSetSizeEx;
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
        };

        const QUOTA_TRIM_HARD: u32 = 0x10000 | 0x40000;

        for ound in 0..ounds {
            
            unsafe {
                let h = GetCurrentProcess();
                EmptyWorkingSet(h);
                SetProcessWorkingSetSizeEx(h, usize::MAX, usize::MAX, QUOTA_TRIM_HARD);
                EmptyWorkingSet(h);
            }

            
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if snapshot != INVALID_HANDLE_VALUE {
                    let mut entry: PROCESSENTRY32W = std::mem::zeroed();
                    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                    if Process32FirstW(snapshot, &mut entry) != 0 {
                        loop {
                            let pid = entry.th32ProcessID;
                            
                            if pid > 4 {
                                let handle = OpenProcess(
                                    PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA,
                                    0,
                                    pid,
                                );
                                if !handle.is_null() {
                                    
                                    EmptyWorkingSet(handle);
                                    
                                    SetProcessWorkingSetSizeEx(
                                        handle,
                                        usize::MAX,
                                        usize::MAX,
                                        QUOTA_TRIM_HARD,
                                    );
                                    
                                    EmptyWorkingSet(handle);
                                    CloseHandle(handle);
                                }
                            }
                            if Process32NextW(snapshot, &mut entry) == 0 {
                                break;
                            }
                        }
                    }
                    CloseHandle(snapshot);
                }
            }

            
            unsafe {
                let h = GetCurrentProcess();
                EmptyWorkingSet(h);
                SetProcessWorkingSetSizeEx(h, usize::MAX, usize::MAX, QUOTA_TRIM_HARD);
                EmptyWorkingSet(h);
            }

            if ound < ounds - 1 {
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
        }
    }

    
    
    
    
    
    
    
    
    
    
    
    
    pub fn optimize(deep: bool) {
        let (has_quota, has_profile) = acquie_pivileges();

        
        if has_quota {
            flush_file_cache_ex();
        }
        if has_profile {
            memory_list_opeation(MEMORY_EMPTY_WORKING_SETS);
            memory_list_opeation(MEMORY_FLUSH_MODIFIED_LIST);
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_SECOND_PRIORITY_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_MODIFIED_PAGE_LIST);
        } else {
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
        }

        
        
        tim_all_processes(2);
        std::thread::sleep(std::time::Duration::from_millis(100));

        
        if has_profile {
            memory_list_opeation(MEMORY_PURGE_STANDBY_LIST);
            memory_list_opeation(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST);
        }

        
        tim_all_processes(2);
        std::thread::sleep(std::time::Duration::from_millis(100));

        
        ty_nt_system_opeations();

        
        let exta_ounds = if deep { 6 } else { 2 };
        tim_all_processes(exta_ounds);

        if deep {
            std::thread::sleep(std::time::Duration::from_millis(150));
            
            ty_nt_system_opeations();
            
            tim_all_processes(3);
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            ty_nt_system_opeations();
            tim_all_processes(2);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod windows_impl {
    pub fn optimize(_deep: bool) {}
    pub fn optimize_best() {}
    pub fn optimize_silent() {}
}

pub use windows_impl::*;
