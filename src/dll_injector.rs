use argh::FromArgs;
use std::error::Error;
use std::ffi::{CString, c_void};
use std::path::Path;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx,
    VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE, WaitForSingleObject,
};
use windows::core::PCSTR;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type ThreadProc = unsafe extern "system" fn(*mut c_void) -> u32;

/// DLL Injector - Inject a DLL into a running process
#[derive(FromArgs)]
struct Args {
    /// target process (PID or process name, e.g., "1234" or "notepad.exe")
    #[argh(positional)]
    process: String,

    /// path to the DLL to inject
    #[argh(positional)]
    dll_path: String,
}

fn process_name_from_entry(entry: &PROCESSENTRY32) -> String {
    let bytes: Vec<u8> = entry
        .szExeFile
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn find_process_by_name(name: &str) -> Result<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| format!("failed to create process snapshot: {e}"))?;

        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };

        Process32First(snapshot, &mut entry)
            .map_err(|e| format!("failed to enumerate processes: {e}"))?;

        let target = name.to_lowercase();

        loop {
            if process_name_from_entry(&entry).to_lowercase() == target {
                let pid = entry.th32ProcessID;
                CloseHandle(snapshot).ok();
                return Ok(pid);
            }

            if Process32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        CloseHandle(snapshot).ok();
        Err(format!("process '{name}' not found").into())
    }
}

fn inject_dll(pid: u32, dll_path: &str) -> Result<()> {
    unsafe {
        if !Path::new(dll_path).exists() {
            return Err(format!("DLL not found: {dll_path}").into());
        }

        let full_path = std::fs::canonicalize(dll_path)
            .map_err(|e| format!("failed to resolve path: {e}"))?;

        let path_str = full_path.to_str().ok_or("invalid path encoding")?;

        let path_cstr =
            CString::new(path_str).map_err(|_| "invalid DLL path")?;

        let process = OpenProcess(
            PROCESS_CREATE_THREAD
                | PROCESS_QUERY_INFORMATION
                | PROCESS_VM_OPERATION
                | PROCESS_VM_WRITE
                | PROCESS_VM_READ,
            false,
            pid,
        )
        .map_err(|e| format!("failed to open process {pid}: {e}"))?;

        let buffer = VirtualAllocEx(
            process,
            None,
            path_cstr.as_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if buffer.is_null() {
            CloseHandle(process).ok();
            return Err("failed to allocate memory in target process".into());
        }

        WriteProcessMemory(
            process,
            buffer,
            path_cstr.as_bytes_with_nul().as_ptr() as *const _,
            path_cstr.as_bytes_with_nul().len(),
            None,
        )
        .map_err(|e| {
            VirtualFreeEx(process, buffer, 0, MEM_RELEASE).ok();
            CloseHandle(process).ok();
            format!("failed to write to process memory: {e}")
        })?;

        let kernel32 =
            GetModuleHandleA(PCSTR(c"kernel32.dll".as_ptr() as *const u8))
                .map_err(|e| format!("failed to get kernel32 handle: {e}"))?;

        let load_library = GetProcAddress(
            kernel32,
            PCSTR(c"LoadLibraryA".as_ptr() as *const u8),
        )
        .ok_or("failed to get LoadLibraryA address")?;

        let thread = CreateRemoteThread(
            process,
            None,
            0,
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                ThreadProc,
            >(load_library)),
            Some(buffer),
            0,
            None,
        )
        .map_err(|e| format!("failed to create remote thread: {e}"))?;

        println!("DLL injection initiated");
        println!("Waiting for LoadLibrary to complete...");

        WaitForSingleObject(thread, 5000);

        CloseHandle(thread).ok();
        VirtualFreeEx(process, buffer, 0, MEM_RELEASE).ok();
        CloseHandle(process).ok();

        println!("Injection complete");
        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let pid = args.process.parse::<u32>().unwrap_or_else(|_| {
        println!("Looking for process: {}", args.process);
        find_process_by_name(&args.process).unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            std::process::exit(1);
        })
    });

    println!("Target PID: {pid}");
    inject_dll(pid, &args.dll_path)
}
