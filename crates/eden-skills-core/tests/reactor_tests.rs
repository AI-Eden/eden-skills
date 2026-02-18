use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eden_skills_core::reactor::{ReactorError, SkillReactor, DEFAULT_CONCURRENCY_LIMIT};
use tokio_util::sync::CancellationToken;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reactor_respects_bounded_concurrency_limit() {
    let reactor = SkillReactor::new(2).expect("reactor");
    let tasks = vec![0usize, 1, 2, 3, 4, 5];

    let current = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));

    let outcomes = reactor
        .run_phase_a(tasks, {
            let current = Arc::clone(&current);
            let max_seen = Arc::clone(&max_seen);
            move |idx| {
                let current = Arc::clone(&current);
                let max_seen = Arc::clone(&max_seen);
                async move {
                    let in_flight = current.fetch_add(1, Ordering::SeqCst) + 1;
                    max_seen.fetch_max(in_flight, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    current.fetch_sub(1, Ordering::SeqCst);
                    Ok::<usize, ReactorError>(idx)
                }
            }
        })
        .await
        .expect("phase a");

    assert_eq!(outcomes.len(), 6);
    assert!(
        max_seen.load(Ordering::SeqCst) <= 2,
        "observed concurrency {} exceeds limit 2",
        max_seen.load(Ordering::SeqCst)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reactor_enforces_two_phase_barrier() {
    let reactor = SkillReactor::new(3).expect("reactor");
    let tasks = vec![0usize, 1, 2];
    let events = Arc::new(Mutex::new(Vec::new()));

    let outcomes = reactor
        .run_two_phase(
            tasks,
            {
                let events = Arc::clone(&events);
                move |idx| {
                    let events = Arc::clone(&events);
                    async move {
                        events
                            .lock()
                            .expect("events lock")
                            .push(format!("a-start-{idx}"));
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        events
                            .lock()
                            .expect("events lock")
                            .push(format!("a-end-{idx}"));
                        Ok::<usize, ReactorError>(idx)
                    }
                }
            },
            {
                let events = Arc::clone(&events);
                move |_phase_a| async move {
                    events
                        .lock()
                        .expect("events lock")
                        .push("b-start".to_string());
                    Ok::<(), ReactorError>(())
                }
            },
        )
        .await
        .expect("run two phase");

    assert_eq!(outcomes.len(), 3);

    let events = events.lock().expect("events lock");
    let b_start = events
        .iter()
        .position(|e| e == "b-start")
        .expect("phase b start marker");
    let last_a_end = events
        .iter()
        .rposition(|e| e.starts_with("a-end-"))
        .expect("phase a end markers");

    assert!(
        b_start > last_a_end,
        "phase b started before phase a completed: {:?}",
        *events
    );
}

#[tokio::test]
async fn reactor_converts_spawn_blocking_panic_to_structured_error() {
    let reactor = SkillReactor::new(DEFAULT_CONCURRENCY_LIMIT).expect("reactor");
    let err = reactor
        .run_blocking::<(), ReactorError, _>("panic-case", || -> Result<(), ReactorError> {
            panic!("boom");
        })
        .await
        .expect_err("panic should map to error");

    assert!(matches!(err, ReactorError::BlockingTaskPanicked { .. }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reactor_supports_cancellation_with_partial_results() {
    let reactor = SkillReactor::new(2).expect("reactor");
    let tasks = vec![0usize, 1, 2, 3, 4, 5];
    let token = CancellationToken::new();
    let cancel_token = token.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(120)).await;
        cancel_token.cancel();
    });

    let (outcomes, cancelled) = reactor
        .run_phase_a_with_cancellation(tasks, token, move |idx| async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            Ok::<usize, ReactorError>(idx)
        })
        .await
        .expect("reactor cancellation run");

    assert!(cancelled, "cancellation flag should be true");
    assert!(
        outcomes.len() < 6,
        "expected partial outcomes after cancellation, got {}",
        outcomes.len()
    );
    assert!(
        !outcomes.is_empty(),
        "expected at least one completed outcome before cancellation"
    );
}
