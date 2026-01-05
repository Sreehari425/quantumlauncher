#[macro_export]
macro_rules! eeprintln {
    ($($arg:tt)*) => {{
        if *$crate::print::IS_GIT_BASH {
            println!("{}", format_args!($($arg)*));
        } else {
            eprintln!("{}", format_args!($($arg)*));
        }
    }};
}

/// Print an informational message.
/// Saved to a log file.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            println!("{} {}", owo_colors::OwoColorize::yellow(&"[info]"), format_args!($($arg)*));
        }
        $crate::print::print_to_file(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Info);
    }};
}

/// Print an informational message.
/// Not saved to a log file.
#[macro_export]
macro_rules! info_no_log {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            println!("{} {}", owo_colors::OwoColorize::yellow(&"[info]"), format_args!($($arg)*));
        }
        $crate::print::print_to_memory(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Info);
    }};
}

/// Print an error message.
/// Not saved to a log file.
#[macro_export]
macro_rules! err_no_log {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            $crate::eeprintln!("{} {}", owo_colors::OwoColorize::red(&"[error]"), format_args!($($arg)*));
        }
        $crate::print::print_to_memory(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Error);
    }};
}

/// Print an error message.
/// Saved to a log file.
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            $crate::eeprintln!("{} {}", owo_colors::OwoColorize::red(&"[error]"), format_args!($($arg)*));
        }
        $crate::print::print_to_file(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Error);
    }};
}

/// Print a point message, i.e. a small step in some process.
/// Saved to a log file.
#[macro_export]
macro_rules! pt {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            println!("{} {}", owo_colors::OwoColorize::bold(&"-"), format_args!($($arg)*));
        }
        $crate::print::print_to_file(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Point);
    }};
}

/// Print a point message, i.e. a small step in some process.
/// Not saved to a log file.
#[macro_export]
macro_rules! pt_no_log {
    ($($arg:tt)*) => {{
        if $crate::print::is_print() {
            println!("{} {}", owo_colors::OwoColorize::bold(&"-"), format_args!($($arg)*));
        }
        $crate::print::print_to_memory(&format!("{}", format_args!($($arg)*)), $crate::print::LogType::Point);
    }};
}
