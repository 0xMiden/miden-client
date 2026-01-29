use std::future::Future;

use indicatif::{ProgressBar, ProgressStyle};

/// Creates a spinner with a message
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner} {msg}")
            .expect("template should be valid"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Runs an async function with a spinner, finishing with a checkmark
pub async fn with_spinner<F, Fut, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let spinner = create_spinner(message);
    let result = f().await;
    spinner.finish_with_message(format!("{message} ✓"));
    result
}
