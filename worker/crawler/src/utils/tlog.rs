use std::sync::atomic::{AtomicUsize, Ordering};

use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset, macros::format_description};

static QUERY_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn local_wall_time(utc: OffsetDateTime) -> PrimitiveDateTime {
    let timestamp =
        jiff::Timestamp::from_second(utc.unix_timestamp()).expect("system time is in range");
    let zoned = timestamp.to_zoned(jiff::tz::TimeZone::system());
    let offset =
        UtcOffset::from_whole_seconds(zoned.offset().seconds()).expect("local offset is in range");
    let local = utc.to_offset(offset);
    PrimitiveDateTime::new(local.date(), local.time())
}

pub fn log_with_timestamp(args: std::fmt::Arguments) {
    let current_time = local_wall_time(OffsetDateTime::now_utc());
    let query_number = QUERY_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!(
        "[{}] [Number #{}] {}",
        current_time
            .format(format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .expect("format describes a primitive date and time"),
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
