use std::{
    ptr::null_mut,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use ctru::prelude::Gfx;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref panicking: AtomicBool = AtomicBool::new(false);
    pub static ref main_id: Arc<Mutex<u32>> = Arc::new(Mutex::new(u32::MAX));
    pub static ref panic_thread: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("main")));
    pub static ref panic_reason: Arc<Mutex<String>> =
        Arc::new(Mutex::new(String::from("No error reported")));
}

pub fn do_panic(run_panic: bool) {
    // this looks extremely weird but this will always end with gfx = Err(ctru::Error::ServiceAlreadyActive) unless poisoned
    let _gfx = Gfx::new();
    let gfx = Gfx::new();
    // memory safety goes out of the window during a panic,
    // This is just copied verbatim from the old C++ and tweaked so that rust accepts it
    unsafe {
        if let Err(ctru::Error::ServiceAlreadyActive) = gfx {
            ctru_sys::gfxSetWide(true);
            ctru_sys::consoleSelect(ctru_sys::consoleInit(ctru_sys::GFX_TOP, null_mut()));

            println!("\x1b[44;37m");
            ctru_sys::consoleClear();
            println!(":<");
            println!();
            println!("The program has encountered an error and cannot continue.");
            println!();
            let panic_thread_guard = panic_thread.lock().unwrap();
            println!("Thread: {}", panic_thread_guard);
            drop(panic_thread_guard);
            println!();
            let panic_reason_guard = panic_reason.lock().unwrap();
            println!("Reason: {}", panic_reason_guard);
            drop(panic_reason_guard);
            println!();
            println!("Press start to exit...");

            while ctru_sys::aptMainLoop() {
                ctru_sys::hidScanInput();

                let k_down = ctru_sys::hidKeysDown();
                if k_down & ctru_sys::KEY_START != 0 {
                    break;
                }

                // gspWaitForVBlank() but ctru_sys doesn't define it because its a macro
                ctru_sys::gspWaitForEvent(ctru_sys::GSPGPU_EVENT_VBlank0, true);
            }
        }
    }
    if run_panic {
        panic!("Non-std thread panicked")
    }
}
