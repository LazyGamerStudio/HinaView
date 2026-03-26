// updater/src/process.rs
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub fn wait_for_process_exit(exe_name: &str) {
    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    let mut warned = false;

    while start.elapsed() < timeout {
        if !is_process_running(exe_name) {
            return;
        }

        if !warned && start.elapsed() > Duration::from_secs(2) {
            println!("[Updater] HinaView is still running. Please close HinaView.exe to continue the update...");
            println!("[Updater] 프로그램이 아직 실행 중입니다. 업데이트를 계속하려면 HinaView.exe를 종료해 주세요...");
            println!("[Updater] プログラムがまだ実行中です. アップデートを続行するにはHinaView.exeを終了してください...");
            println!("[Updater] 程序仍在运行. 请关闭 HinaView.exe 以继续更新...");
            println!("[Updater] 程式仍在執行中. 請關閉 HinaView.exe 以繼續更新...");
            warned = true;
        }

        thread::sleep(Duration::from_millis(500));
    }
}

pub fn is_process_running(exe_name: &str) -> bool {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("updater.exe"));
    let parent_dir = current_exe.parent().unwrap_or(Path::new("."));
    let exe_path = parent_dir.join(exe_name);

    if !exe_path.exists() {
        return false;
    }

    std::fs::OpenOptions::new()
        .write(true)
        .open(&exe_path)
        .is_err()
}
