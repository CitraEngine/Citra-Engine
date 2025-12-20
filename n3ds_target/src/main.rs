use std::{
    cell::RefCell,
    panic::{self, PanicHookInfo},
    rc::Rc,
    sync::atomic::Ordering,
};

use citra_engine::{Engine, script::ScriptLibrary};
use ctru::{
    prelude::{Apt, Gfx},
    services::gfx::{TopScreen, TopScreen3D},
    //services::soc::Soc,
};
use ctru_sys::{CUR_THREAD_HANDLE, svcGetThreadId, threadExit};
use n3ds_target::{
    asset_provider::N3dsAssetProvider,
    audio::N3dsAudioManager,
    input::N3dsInputManager,
    panicking::{do_panic, main_id, panic_reason, panic_thread, panicking},
    render::N3dsGraphicsManager,
};
use scripts::Scripts;

fn bsod_panic(info: &PanicHookInfo) {
    // set panic reason
    let mut panic_reason_guard = panic_reason.lock().unwrap();
    *panic_reason_guard = info.to_string();
    drop(panic_reason_guard);
    // Since the audio thread is created through non-rust-std means
    // panic does not work correctly.
    // we need to check if we are on the main thread
    // if not, send a signal to the main thread to also panic
    panicking.store(true, Ordering::SeqCst);
    let main_id_guard = main_id.lock().unwrap();
    let m_id = *main_id_guard;
    drop(main_id_guard);
    if m_id != u32::MAX {
        let mut id = u32::MAX;
        let _ = unsafe { svcGetThreadId(&mut id, CUR_THREAD_HANDLE) };
        if id != m_id {
            let mut panic_thread_guard = panic_thread.lock().unwrap();
            *panic_thread_guard = String::from("audio");
            drop(panic_thread_guard);
            unsafe { threadExit(0) };
        }
    }
    // by this time we know this code is executed by the main thread
    let mut panic_thread_guard = panic_thread.lock().unwrap();
    *panic_thread_guard = "main".to_owned();
    drop(panic_thread_guard);
    do_panic(false);
}

fn main() {
    panic::set_hook(Box::new(bsod_panic));
    /*#[cfg(debug_assertions)]
    {
        let mut soc = Soc::new().expect("Unable to get SOC");
        let _ = soc.redirect_to_3dslink(true, true); // disregard and continue if fail
    }*/
    let mut apt = Apt::new().unwrap();
    let mut main_thread_id = u32::MAX;
    let _ = unsafe { svcGetThreadId(&mut main_thread_id, CUR_THREAD_HANDLE) };
    if main_thread_id == u32::MAX {
        panic!("Main thread id invalid (U32_MAX)");
    }
    let mut main_thread_id_guard = main_id.lock().unwrap();
    *main_thread_id_guard = main_thread_id;
    drop(main_thread_id_guard);
    apt.set_app_cpu_time_limit(50).unwrap();
    Engine::register_scripts(Scripts::get_scripts());
    let gfx = Box::leak::<'static>(Box::new(Gfx::new().expect("Could not initialize GFX")));
    let top_screen: &'static RefCell<TopScreen> = &gfx.top_screen;
    let top_screen = Box::leak::<'static>(Box::new(TopScreen3D::from(top_screen)));
    let mut engine = Engine::new(
        Rc::new(RefCell::new(
            N3dsAssetProvider::new().expect("Error while starting asset provider"),
        )),
        Rc::new(RefCell::new(N3dsAudioManager::new().unwrap())),
        Box::new(
            N3dsGraphicsManager::new(gfx, top_screen)
                .expect("Error while starting graphics manager"),
        ),
        Rc::new(RefCell::new(
            N3dsInputManager::new(apt).expect("Error while starting input manager"),
        )),
    );
    engine.run();
}
