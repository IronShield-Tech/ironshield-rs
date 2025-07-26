use tokio::task::JoinHandle;
use futures::future;

use ironshield_types::{IronShieldChallenge, IronShieldChallengeResponse};
use crate::config::ClientConfig;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use tokio::time::Duration;
use crate::error::ErrorHandler;
use crate::result::ResultHandler;

/// Configuration for proof-of-work challenge
/// solving.
///
/// * `thread_count`:      Number of threads to use
///                        for solving.
/// * `use_multithreaded`: Whether to use
///                        multithreaded solving
#[derive(Debug, Clone)]
pub struct SolveConfig {
    pub thread_count:      usize,
    pub use_multithreaded: bool,
}

impl SolveConfig {
    /// Creates a new solve configuration based on system
    /// capabilities and user preference.
    ///
    /// # Arguments
    /// * `config`:            Client configuration containing
    ///                        optional thread count override.
    /// * `use_multithreaded`: Whether to enable multithreaded
    ///                        solving.
    ///
    /// # Returns
    /// * `Self`: A new instance of the solving config.
    pub fn new(config: &ClientConfig, use_multithreaded: bool) -> Self {
        let available_cores: usize = num_cpus::get();

        // Use 80% of available cores, minimum 1, respect config override.
        let thread_count: usize = if use_multithreaded {
            config.num_threads
                .unwrap_or_else(|| std::cmp::max(1, (available_cores * 4) / 5))
        } else {
            1
        };

        Self {
            thread_count,
            use_multithreaded,
        }
    }
}

/// Trait for progress callbacks during solving
pub trait ProgressTracker: Send + Sync {
    fn on_progress(&self, thread_id: usize, total_attempts: u64, hash_rate: u64, elapsed: std::time::Duration);
}

/// Primary entry point for solving proof-of-work challenges.
///
/// # Arguments
/// * `challenge`:          The challenge to solve.
/// * `config`:             Client configuration. `ClientConfig`
/// * `use_multithreading`: Whether to attempt multithreaded solving.
/// * `progress_tracker`:   Optional progress tracker for detailed logging
///
/// # Returns
/// `ResultHandler<IronShieldChallengeResponse>`: A valid solution:
///                                               `Ok(IronShieldChallengeResponse)`
///                                               or an error:
///                                               `Err(ErrorHandler)`.
pub async fn solve_challenge(
    challenge:         IronShieldChallenge,
    config:            &ClientConfig,
    use_multithreaded: bool,
    progress_tracker:  Option<Arc<dyn ProgressTracker>>,
) -> ResultHandler<IronShieldChallengeResponse> {
    let solve_config: SolveConfig = SolveConfig::new(config, use_multithreaded);

    let _start_time: Instant = Instant::now();

    // Choose a solving strategy based on configuration.
    let result = if solve_config.use_multithreaded && solve_config.thread_count > 1 {
        solve_multithreaded(challenge, &solve_config, config, progress_tracker).await
    } else {
        solve_single_threaded(challenge, config).await
    };

    // Return result without logging
    result
}

/// Solve using multiple threads with early termination when a solution is found.
async fn solve_multithreaded(
    challenge: IronShieldChallenge,
    solve_config: &SolveConfig,
    config: &ClientConfig,
    progress_tracker: Option<Arc<dyn ProgressTracker>>,
) -> ResultHandler<IronShieldChallengeResponse> {
    let challenge: Arc<IronShieldChallenge> = Arc::new(challenge);
    let solution_found: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<JoinHandle<Result<IronShieldChallengeResponse, ErrorHandler>>> = Vec::new();

    // Spawn worker threads with proper stride and offset.
    for thread_id in 0..solve_config.thread_count {
        let      challenge_clone: Arc<IronShieldChallenge> = Arc::clone(&challenge);
        let        thread_stride: u64 = solve_config.thread_count as u64;
        let        thread_offset: u64 = thread_id as u64;
        let         config_clone: ClientConfig = config.clone();
        let solution_found_clone: Arc<AtomicBool> = Arc::clone(&solution_found);
        let progress_tracker_clone = progress_tracker.clone();

        let handle = tokio::task::spawn_blocking(move || {
            // Create progress callback for status updates.
            let core_progress_callback = create_progress_callback(
                thread_id,
                config_clone.clone(),
                solution_found_clone,
                progress_tracker_clone,
            );

            // Call ironshield-core's find_solution_multi_threaded function.
            ironshield_core::find_solution_multi_threaded(
                &*challenge_clone,
                Some(ironshield_core::PoWConfig::multi_threaded()), // Use optimized multithreaded config
                Some(thread_offset as usize),                       // start_offset for this thread.
                Some(thread_stride as usize),                       // stride for optimal thread-stride pattern.
                Some(&core_progress_callback),                      // Progress callback for status updates.
            ).map_err(|e: String| ErrorHandler::ProcessingError(format!(
                "Thread {} failed: {}", thread_id, e
            )))
        });

        handles.push(handle);
    }

    // Wait for ANY thread to find a solution and immediately signal others to stop.
    wait_for_solution(handles, solution_found, config).await
}

/// Create a progress callback for a worker thread.
fn create_progress_callback(
    thread_id: usize,
    _config: ClientConfig,
    solution_found: Arc<AtomicBool>,
    progress_tracker: Option<Arc<dyn ProgressTracker>>,
) -> impl Fn(u64) {
    let thread_start_time: Instant = Instant::now();
    let cumulative_attempts: Arc<std::sync::atomic::AtomicU64> = Arc::new(std::sync::atomic::AtomicU64::new(0));

    move |batch_attempts: u64| {
        // Stop reporting progress if a solution already found by another thread.
        if solution_found.load(Ordering::Relaxed) {
            return;
        }

        // Accumulate attempts (core callback provides batch size, not cumulative).
        let total_attempts: u64 = cumulative_attempts.fetch_add(batch_attempts, Ordering::Relaxed) + batch_attempts;

        // Progress tracking
        let _elapsed: Duration = thread_start_time.elapsed();
        let _elapsed_millis: u64 = _elapsed.as_millis() as u64;

        // Calculate hash rate based on cumulative attempts.
        let _hash_rate: u64 = if _elapsed_millis > 0 {
            (total_attempts * 1000) / _elapsed_millis
        } else {
            total_attempts  // If solved instantly, assume 1ms.
        };

        // Progress information is available here but not currently logged
        // The CLI wrapper will handle progress display through animations

        // Call the provided progress callback if it exists
        if let Some(tracker) = &progress_tracker {
            tracker.on_progress(thread_id, total_attempts, _hash_rate, _elapsed);
        }
    }
}

/// Wait for any thread to find a solution and abort remaining threads.
async fn wait_for_solution(
    mut handles:    Vec<JoinHandle<ResultHandler<IronShieldChallengeResponse>>>,
    solution_found: Arc<AtomicBool>,
    _config:        &ClientConfig,
) -> ResultHandler<IronShieldChallengeResponse> {
    while !handles.is_empty() {
        // Wait for the first handle to complete.
        let (result, _thread_index, other_handles) = future::select_all(handles).await;

        match result {
            Ok(Ok(found_solution)) => {
                // Signal all threads to stop progress reporting.
                solution_found.store(true, Ordering::Relaxed);

                // Abort all remaining handles immediately.
                for handle in other_handles {
                    handle.abort();
                }

                return Ok(found_solution);
            },
            Ok(Err(_e)) => {
                // Continue with remaining threads
                handles = other_handles;
            },
            Err(_e) => {
                // Continue with remaining threads
                handles = other_handles;
            }
        }
    }

    Err(ErrorHandler::ProcessingError(
        "No solution found by any thread".to_string()
    ))
}

/// Solve using a single thread.
async fn solve_single_threaded(
    challenge: IronShieldChallenge,
    _config: &ClientConfig,
) -> ResultHandler<IronShieldChallengeResponse> {
    // Use tokio::task::spawn_blocking to avoid blocking the async runtime.
    let handle = tokio::task::spawn_blocking(move || {
        // Use single-threaded function (progress callbacks not supported in single-threaded core).
        ironshield_core::find_solution_single_threaded(&challenge, Some(ironshield_core::PoWConfig::single_threaded()))
    });

    match handle.await {
        Ok(Ok(solution)) => {
            Ok(solution)
        },
        Ok(Err(e)) => {
            Err(ErrorHandler::ProcessingError(format!(
                "Single-threaded solve failed: {}", e
            )))
        },
        Err(e) => {
            Err(ErrorHandler::ProcessingError(format!(
                "Single-threaded solve task failed: {}", e
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_solve_config_single_threaded() {
        let config = ClientConfig {
            api_base_url: "https://api.test.com".to_string(),
            num_threads: Some(4),
            timeout: Duration::from_secs(30),
            user_agent: crate::constant::USER_AGENT.to_string(),
            verbose: false,
        };

        let solve_config = SolveConfig::new(&config, false);
        assert_eq!(solve_config.thread_count, 1);
        assert!(!solve_config.use_multithreaded);
    }

    #[test]
    fn test_solve_config_multithreaded() {
        let config = ClientConfig {
            api_base_url: "https://api.test.com".to_string(),
            num_threads: Some(4),
            timeout: Duration::from_secs(30),
            user_agent: crate::constant::USER_AGENT.to_string(),
            verbose: false,
        };

        let solve_config = SolveConfig::new(&config, true);
        assert_eq!(solve_config.thread_count, 4);
        assert!(solve_config.use_multithreaded);
    }

    #[test]
    fn test_solve_config_auto_thread_count() {
        let config = ClientConfig {
            api_base_url: "https://api.test.com".to_string(),
            num_threads: None, // Auto-detect.
            timeout: Duration::from_secs(30),
            user_agent: crate::constant::USER_AGENT.to_string(),
            verbose: false,
        };

        let solve_config = SolveConfig::new(&config, true);
        assert!(solve_config.thread_count >= 1);
        assert!(solve_config.use_multithreaded);
    }
} 