use std::panic;

pub fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(message) = panic_info.message() {
            error!("panic: {}", message);
        } else if let Some(payload) = panic_info.payload().downcast_ref::<&'static str>() {
            error!("panic: {}", payload);
        } else {
            error!("panic");
        }
    }));
}
