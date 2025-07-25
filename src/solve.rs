use tokio::task::JoinHandle;
use futures::future;

use ironshield_api::handler::{error::ErrorHandler, result::ResultHandler};
use ironshield_types::{IronShieldChallenge, IronShieldChallengeResponse};
use crate::config::ClientConfig;
use crate::display::{ProgressAnimation, format_number_with_commas};

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use tokio::time::Duration;

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
        let available_cores = num_cpus::get();

        // Use 80% of available cores, minimum 1, respect config override.
        let thread_count = if use_multithreaded {
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

/// Primary entry point for solving proof-of-work challenges.
///
/// # Arguments
/// * `challenge`:          The challenge to solve.
/// * `config`:             Client configuration. `ClientConfig`
/// * `use_multithreading`: Whether to attempt multithreaded solving.
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
) -> ResultHandler<IronShieldChallengeResponse> {
    let solve_config = SolveConfig::new(config, use_multithreaded);

    // Log configuration details.
    crate::verbose_section!(config, "Challenge Solving");
    crate::verbose_kv!(config, "Thread Count", solve_config.thread_count);
    crate::verbose_kv!(config, "Multithreaded", solve_config.use_multithreaded);
    crate::verbose_kv!(config, "Recommended Attempts", challenge.recommended_attempts);

    // Always show challenge difficulty info (both verbose and non-verbose modes)
    let difficulty: u64 = challenge.recommended_attempts / 2; // recommended_attempts = difficulty * 2
    println!("Received proof-of-work challenge with difficulty {}", format_number_with_commas(difficulty));

    let start_time: Instant = Instant::now();
    
    // Start the progress animation (only in non-verbose mode)
    let animation = ProgressAnimation::new(config.verbose);
    let animation_handle = animation.start();

    // Choose solving strategy based on configuration.
    let result = if solve_config.use_multithreaded && solve_config.thread_count > 1 {
        solve_multithreaded(challenge, &solve_config, config).await
    } else {
        solve_single_threaded(challenge, config).await
    };

    // Stop the animation and clean up the line
    animation.stop(animation_handle).await;

    // Log timing and performance metrics.
    match result {
        Ok(solution) => {
            log_solution_performance(&solution, start_time.elapsed(), &solve_config, config);
            Ok(solution)
        },
        Err(e) => {
            crate::verbose_log!(
                config,
                error,
                "Challenge solving failed after {:?}: {}",
                start_time.elapsed(),
                e
            );
            Err(e)
        }
    }
}

/// Log performance metrics for a solved challenge.
fn log_solution_performance(
    solution:     &IronShieldChallengeResponse,
    elapsed:      std::time::Duration,
    solve_config: &SolveConfig,
    config:       &ClientConfig,
) {
    let elapsed_millis: u64 = elapsed.as_millis() as u64;

    // Calculate estimated total attempts across all threads using thread-stride analysis.
    // In thread-stride: if thread T finds solution at nonce N, it has done roughly (N/thread_count) attempts.
    // Other threads have done roughly the same amount of work.
    let solution_nonce: u64 = solution.solution as u64;
    let estimated_attempts_per_thread: u64 = (solution_nonce / solve_config.thread_count as u64) + 1;
    let estimated_total_attempts: u64 = estimated_attempts_per_thread * solve_config.thread_count as u64;

    let hash_rate: u64 = if elapsed_millis > 0 {
        (estimated_total_attempts * 1000) / elapsed_millis
    } else {
        estimated_total_attempts  // If solved instantly, assume 1ms.
    };

    crate::verbose_log!(
        config,
        timing,
        "Challenge solved in {:?} (~{} estimated total attempts, ~{} h/s)",
        elapsed,
        estimated_total_attempts,
        hash_rate
    );

    crate::verbose_log!(
        config,
        success,
        "Performance: {} threads achieved ~{} hashes/second (solution found at nonce {})",
        solve_config.thread_count,
        hash_rate,
        solution_nonce
    );
}

/// Solve using multiple threads with early termination when solution is found.
async fn solve_multithreaded(
    challenge: IronShieldChallenge,
    solve_config: &SolveConfig,
    config: &ClientConfig,
) -> ResultHandler<IronShieldChallengeResponse> {
    crate::verbose_log!(config, compute, "Starting multithreaded solve with {} threads", solve_config.thread_count);

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

        let handle = tokio::task::spawn_blocking(move || {
            // Create progress callback for status updates.
            let progress_callback = create_progress_callback(
                thread_id,
                config_clone.clone(),
                solution_found_clone
            );

            // Call ironshield-core's find_solution_multi_threaded function.
            ironshield_core::find_solution_multi_threaded(
                &*challenge_clone,
                Some(ironshield_core::PoWConfig::multi_threaded()), // Use optimized multi-threaded config
                Some(thread_offset as usize),      // start_offset for this thread.
                Some(thread_stride as usize),      // stride for optimal thread-stride pattern.
                Some(&progress_callback),          // Progress callback for status updates.
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
    config: ClientConfig,
    solution_found: Arc<AtomicBool>,
) -> impl Fn(u64) {
    let thread_start_time: Instant = Instant::now();
    let cumulative_attempts: Arc<std::sync::atomic::AtomicU64> = Arc::new(std::sync::atomic::AtomicU64::new(0));

    move |batch_attempts: u64| {
        // Stop reporting progress if solution already found by another thread.
        if solution_found.load(Ordering::Relaxed) {
            return;
        }

        // Accumulate attempts (core callback provides batch size, not cumulative).
        let total_attempts: u64 = cumulative_attempts.fetch_add(batch_attempts, Ordering::Relaxed) + batch_attempts;

        let elapsed: Duration = thread_start_time.elapsed();
        let elapsed_millis: u64 = elapsed.as_millis() as u64;

        // Calculate hash rate based on cumulative attempts.
        let hash_rate: u64 = if elapsed_millis > 0 {
            (total_attempts * 1000) / elapsed_millis
        } else {
            total_attempts  // If solved instantly, assume 1ms.
        };

        crate::verbose_log!(
            config,
            compute,
            "Thread {} progress: {} total attempts on this thread ({} hashes/second)",
            thread_id,
            total_attempts,
            hash_rate
        );
    }
}

/// Wait for any thread to find a solution and abort remaining threads.
async fn wait_for_solution(
    mut handles:    Vec<JoinHandle<ResultHandler<IronShieldChallengeResponse>>>,
    solution_found: Arc<AtomicBool>,
    config:         &ClientConfig,
) -> ResultHandler<IronShieldChallengeResponse> {
    while !handles.is_empty() {
        // Wait for the first handle to complete.
        let (result, thread_index, other_handles) = future::select_all(handles).await;

        match result {
            Ok(Ok(found_solution)) => {
                // Signal all threads to stop progress reporting.
                solution_found.store(true, Ordering::Relaxed);

                crate::verbose_log!(
                    config,
                    success,
                    "Thread {} found solution! Signaling {} other threads to stop.",
                    thread_index,
                    other_handles.len()
                );

                // Abort all remaining handles immediately.
                for handle in other_handles {
                    handle.abort();
                }

                return Ok(found_solution);
            },
            Ok(Err(e)) => {
                crate::verbose_log!(
                    config,
                    warning,
                    "Thread {} error: {}. Continuing with {} remaining threads.",
                    thread_index,
                    e,
                    other_handles.len()
                );
                handles = other_handles;
            },
            Err(e) => {
                crate::verbose_log!(
                    config,
                    error,
                    "Thread {} join error: {}. Continuing with {} remaining threads.",
                    thread_index,
                    e,
                    other_handles.len()
                );
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
    config: &ClientConfig,
) -> ResultHandler<IronShieldChallengeResponse> {
    crate::verbose_log!(config, compute, "Starting single-threaded solve");

    // Use tokio::task::spawn_blocking to avoid blocking the async runtime.
    let handle = tokio::task::spawn_blocking(move || {
        // Use single-threaded function (progress callbacks not supported in single-threaded core).
        ironshield_core::find_solution_single_threaded(&challenge, Some(ironshield_core::PoWConfig::single_threaded()))
    });

    match handle.await {
        Ok(Ok(solution)) => {
            crate::verbose_log!(config, success, "Single-threaded solve completed successfully");
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