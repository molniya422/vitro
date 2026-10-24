// Vitro perf harness — M1 验收测量(PLAN §1 硬性目标)
// 空闲 CPU <0.5%(目标 0.0%)、RAM(工作集)≤30MB
#![allow(non_snake_case)]

use std::process::Command;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;

fn filetime_ms(t: FILETIME) -> f64 {
    let v = ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
    v as f64 / 10_000.0
}

fn sample(handle: HANDLE) -> (f64, f64) {
    unsafe {
        let mut _creation = FILETIME::default();
        let mut _exit = FILETIME::default();
        let mut k = FILETIME::default();
        let mut u = FILETIME::default();
        let _ = GetProcessTimes(handle, &mut _creation, &mut _exit, &mut k, &mut u);
        let mut pmc = PROCESS_MEMORY_COUNTERS::default();
        let _ = GetProcessMemoryInfo(handle, &mut pmc, std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32);
        (filetime_ms(k) + filetime_ms(u), pmc.WorkingSetSize as f64 / 1024.0 / 1024.0)
    }
}

fn main() -> Result<()> {
    let exe = std::env::current_exe()?
        .parent()
        .ok_or_else(|| windows::core::Error::from(windows::core::HRESULT(-1)))?
        .join("vitro.exe");
    let mut child = Command::new(exe).spawn().expect("spawn vitro.exe");
    std::thread::sleep(std::time::Duration::from_millis(3000)); // 初始化窗口

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, child.id())?;

        let (cpu0, ram0) = sample(handle);
        std::thread::sleep(std::time::Duration::from_millis(5000));
        let (cpu1, ram1) = sample(handle);

        let cpu_pct = (cpu1 - cpu0) / 5000.0 * 100.0;
        let ram = ram1.max(ram0);
        let ok = cpu_pct < 0.5 && ram <= 30.0;

        println!("═══ Vitro M1 perf harness ═══");
        println!("idle CPU  : {:>6.3} %   (budget < 0.5%, target 0.0%)", cpu_pct);
        println!("RAM (WS)  : {:>6.2} MB  (budget <= 30 MB)", ram);
        println!("verdict   : {}", if ok { "✅ PASS — M1 验收达标" } else { "❌ FAIL" });

        let _ = TerminateProcess(handle, 0);
        drop(handle);
        let _ = child.wait();
        if ok { Ok(()) } else { Err(Error::from_win32()) }
    }
}
