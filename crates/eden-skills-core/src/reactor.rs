use std::future::Future;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub use crate::error::ReactorError;

pub const DEFAULT_CONCURRENCY_LIMIT: usize = 10;
pub const MIN_CONCURRENCY_LIMIT: usize = 1;
pub const MAX_CONCURRENCY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseOutcome<T, E> {
    pub index: usize,
    pub result: Result<T, E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillReactor {
    concurrency_limit: usize,
}

impl Default for SkillReactor {
    fn default() -> Self {
        Self {
            concurrency_limit: DEFAULT_CONCURRENCY_LIMIT,
        }
    }
}

impl SkillReactor {
    pub fn new(concurrency_limit: usize) -> Result<Self, ReactorError> {
        if !(MIN_CONCURRENCY_LIMIT..=MAX_CONCURRENCY_LIMIT).contains(&concurrency_limit) {
            return Err(ReactorError::InvalidConcurrency {
                provided: concurrency_limit,
                min: MIN_CONCURRENCY_LIMIT,
                max: MAX_CONCURRENCY_LIMIT,
            });
        }
        Ok(Self { concurrency_limit })
    }

    pub fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    pub async fn run_phase_a<I, O, E, F, Fut>(
        &self,
        tasks: Vec<I>,
        phase_a: F,
    ) -> Result<Vec<PhaseOutcome<O, E>>, ReactorError>
    where
        I: Send + 'static,
        O: Send + 'static,
        E: Send + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, E>> + Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let phase_a = Arc::new(phase_a);
        let mut join_set = JoinSet::new();

        for (index, task) in tasks.into_iter().enumerate() {
            let semaphore = Arc::clone(&semaphore);
            let phase_a = Arc::clone(&phase_a);
            join_set.spawn(async move {
                let permit =
                    semaphore
                        .acquire_owned()
                        .await
                        .map_err(|_| ReactorError::RuntimeShutdown {
                            context: "failed to acquire phase-a semaphore permit".to_string(),
                        })?;
                let result = phase_a(task).await;
                drop(permit);
                Ok::<PhaseOutcome<O, E>, ReactorError>(PhaseOutcome { index, result })
            });
        }

        let mut outcomes = Vec::new();
        while let Some(joined) = join_set.join_next().await {
            let phase_outcome = joined.map_err(|err| ReactorError::TaskJoin {
                context: "phase-a task join".to_string(),
                detail: err.to_string(),
            })??;
            outcomes.push(phase_outcome);
        }

        outcomes.sort_by_key(|outcome| outcome.index);
        Ok(outcomes)
    }

    pub async fn run_two_phase<I, O, E, F, Fut, PhaseB, PhaseBFut>(
        &self,
        tasks: Vec<I>,
        phase_a: F,
        phase_b: PhaseB,
    ) -> Result<Vec<PhaseOutcome<O, E>>, ReactorError>
    where
        I: Send + 'static,
        O: Send + 'static,
        E: Send + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, E>> + Send + 'static,
        PhaseB: FnOnce(&[PhaseOutcome<O, E>]) -> PhaseBFut + Send,
        PhaseBFut: Future<Output = Result<(), ReactorError>> + Send,
    {
        let phase_a_outcomes = self.run_phase_a(tasks, phase_a).await?;
        phase_b(&phase_a_outcomes).await?;
        Ok(phase_a_outcomes)
    }

    pub async fn run_blocking<R, E, F>(&self, task_name: &str, operation: F) -> Result<R, E>
    where
        R: Send + 'static,
        E: From<ReactorError> + Send + 'static,
        F: FnOnce() -> Result<R, E> + Send + 'static,
    {
        let task_name = task_name.to_string();
        match tokio::task::spawn_blocking(operation).await {
            Ok(result) => result,
            Err(err) if err.is_cancelled() => Err(E::from(ReactorError::BlockingTaskCancelled {
                task: task_name,
            })),
            Err(err) => Err(E::from(ReactorError::BlockingTaskPanicked {
                task: task_name,
                detail: err.to_string(),
            })),
        }
    }
}
