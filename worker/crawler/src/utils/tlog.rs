use chrono::Local;
use std::sync::atomic::{AtomicUsize, Ordering};

static QUERY_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn log_with_timestamp(args: std::fmt::Arguments) {
    let current_time = Local::now();
    let query_number = QUERY_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!(
        "[{}] [Number #{}] {}",
        current_time.format("%Y-%m-%d %H:%M:%S"),
        query_number,
        args
    );
}

#[macro_export]
macro_rules! tlog {
    ($($arg:tt)*) => {
        // log_with_timestamp を呼び出し
        $crate::utils::tlog::log_with_timestamp(format_args!($($arg)*))
    };
}
