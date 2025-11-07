//! Helpers for writing CLIs.

/// Prints a success.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*).green().bold();
        println!("{message}");
    }};
}

/// Prints a warning.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;

        let log_message = ntf_poc_helpers::helpers::uncapitalise(&format!($($arg)*));
        let log_message = log_message.trim_end_matches(".");
        tracing::warn!("{log_message}");

        let message = indoc::formatdoc!($($arg)*).yellow().bold();
        eprintln!("{message}");
    }};
}

/// Prints an error.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*);
        let message = ntf_poc_helpers::helpers::uncapitalise(&message);
        let message = format!("Error: {message}").red().bold();
        eprintln!("{message}");
    }};
}

/// Prints a hint.
#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        let message = indoc::formatdoc!($($arg)*).blue();
        eprintln!("{message}");
    }};
}
