use std::panic::{self, PanicHookInfo};

pub fn install_crash_handler() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_hook_info| {
        crash_handler(panic_hook_info);
        default_hook(panic_hook_info);
    }));
}

fn crash_handler(_panic_hook_info: &PanicHookInfo) {
    let machine_id = std::fs::read_to_string("/etc/machine-id").unwrap_or("unknown".to_string());
    let hwid = machine_id.trim_end();
    let mut stacktrace = _panic_hook_info
        .payload()
        .downcast_ref::<&str>()
        .unwrap_or(&"explicit panic")
        .to_string();
    stacktrace.push('\n');
    stacktrace.push_str(&std::backtrace::Backtrace::force_capture().to_string());

    let json = serde_json::json!({"hwid":hwid,"stacktrace":stacktrace});

    let _ = ureq::post("https://deadlocked.holyhades64.workers.dev/").send_json(json);

    log::error!("crash reported");
}
