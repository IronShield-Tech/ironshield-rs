use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::io::Write;

/// A progress animation that shows a spinning indicator during long-running operations.
/// 
/// The animation only displays when not in verbose mode, allowing for clean output
/// during normal operation while preserving detailed logging when needed.
pub struct ProgressAnimation {
    running: Arc<AtomicBool>,
    verbose: bool,
}

impl ProgressAnimation {
    /// Creates a new progress animation.
    ///
    /// # Arguments
    /// * `verbose` - If true, the animation will not be displayed to avoid interfering with verbose output
    ///
    /// # Returns
    /// * `Self` - A new ProgressAnimation instance
    pub fn new(verbose: bool) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            verbose,
        }
    }

    /// Starts the progress animation if not in verbose mode.
    ///
    /// # Returns
    /// * `Option<JoinHandle<()>>` - A handle to the animation task if started, None if verbose mode
    ///
    /// # Example
    /// ```
    /// let animation = ProgressAnimation::new(false);
    /// let handle = animation.start();
    /// // ... do work ...
    /// animation.stop(handle).await;
    /// ```
    pub fn start(&self) -> Option<JoinHandle<()>> {
        if self.verbose {
            return None;
        }

        self.running.store(true, Ordering::Relaxed);
        let running_clone = Arc::clone(&self.running);
        
        Some(tokio::spawn(async move {
            show_progress_animation(running_clone).await;
        }))
    }

    /// Stops the progress animation and cleans up the display.
    ///
    /// # Arguments
    /// * `handle` - The animation task handle returned from `start()`
    ///
    /// # Example
    /// ```
    /// let animation = ProgressAnimation::new(false);
    /// let handle = animation.start();
    /// // ... do work ...
    /// animation.stop(handle).await;
    /// ```
    pub async fn stop(&self, handle: Option<JoinHandle<()>>) {
        // Signal the animation to stop
        self.running.store(false, Ordering::Relaxed);

        // Wait for the animation task to complete and clean up the line
        if let Some(animation_handle) = handle {
            let _ = animation_handle.await; // Wait for animation to stop
            if !self.verbose {
                print!("\r\x1b[K"); // Clear the animation line
                std::io::stdout().flush().unwrap_or(());
            }
        }
    }
}

/// Shows a simple spinning animation while a long-running operation is in progress.
/// 
/// The animation cycles through different characters to create a spinning effect:
/// | / — \
///
/// # Arguments
/// * `running` - An atomic boolean that controls when the animation should stop
async fn show_progress_animation(running: Arc<AtomicBool>) {
    let mut timer = interval(Duration::from_millis(250));
    let dots_patterns: [&'static str; 4] = ["|", "/", "—", "\\"];
    let mut pattern_index: usize = 0;

    // Skip the first tick (it fires immediately)
    timer.tick().await;

    while running.load(Ordering::Relaxed) {
        print!("\r\x1b[KSolving Challenge {}", dots_patterns[pattern_index]);
        std::io::stdout().flush().unwrap_or(());
        
        pattern_index = (pattern_index + 1) % dots_patterns.len(); 
        
        timer.tick().await;
    }
}

/// Formats a number with comma separators for better readability.
///
/// # Arguments
/// * `num` - The number to format
///
/// # Returns
/// * `String` - The formatted number with comma separators
///
/// # Example
/// ```
/// assert_eq!(format_number_with_commas(1234567), "1,234,567");
/// assert_eq!(format_number_with_commas(42), "42");
/// assert_eq!(format_number_with_commas(1000), "1,000");
/// ```
pub fn format_number_with_commas(num: u64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_with_commas() {
        assert_eq!(format_number_with_commas(0), "0");
        assert_eq!(format_number_with_commas(42), "42");
        assert_eq!(format_number_with_commas(123), "123");
        assert_eq!(format_number_with_commas(1000), "1,000");
        assert_eq!(format_number_with_commas(12345), "12,345");
        assert_eq!(format_number_with_commas(1234567), "1,234,567");
        assert_eq!(format_number_with_commas(1234567890), "1,234,567,890");
    }

    #[test]
    fn test_progress_animation_verbose_mode() {
        let animation = ProgressAnimation::new(true);
        let handle = animation.start();
        assert!(handle.is_none(), "Animation should not start in verbose mode");
    }

    #[tokio::test]
    async fn test_progress_animation_non_verbose_mode() {
        let animation = ProgressAnimation::new(false);
        let handle = animation.start();
        assert!(handle.is_some(), "Animation should start in non-verbose mode");
        
        // Clean up the animation
        animation.stop(handle).await;
    }
} 