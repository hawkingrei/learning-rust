/// Log slow operations with warn!.
macro_rules! slow_log {
    ($t:expr, $($arg:tt)*) => {{
        if $t.is_slow() {
            warn!("{} [takes {:?}]", format_args!($($arg)*), $t.elapsed());
        }
    }}
}

/// make a thread name with additional tag inheriting from current thread.
#[macro_export]
macro_rules! thd_name {
    ($name:expr) => {{
        $crate::util::get_tag_from_thread_name()
            .map(|tag| format!("{}::{}", $name, tag))
            .unwrap_or_else(|| $name.to_owned())
    }};
}
