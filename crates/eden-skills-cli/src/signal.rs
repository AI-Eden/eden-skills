//! Cooperative SIGINT handling for interactive prompts.
//!
//! The process-level Ctrl+C handler can choose between immediate exit and
//! deferred cancellation. During dialoguer prompts we temporarily enter a
//! "prompt-interruptible" region so SIGINT can be translated into a graceful
//! command-level cancel path (for example, `Install cancelled`).

use std::sync::atomic::{AtomicBool, Ordering};

static PROMPT_INTERRUPTIBLE: AtomicBool = AtomicBool::new(false);
static PROMPT_INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Whether SIGINT should be deferred to the current prompt.
pub fn prompt_interruptible() -> bool {
    PROMPT_INTERRUPTIBLE.load(Ordering::SeqCst)
}

/// Record that SIGINT occurred while a prompt was active.
pub fn request_prompt_interrupt() {
    PROMPT_INTERRUPTED.store(true, Ordering::SeqCst);
}

/// Consume and clear the pending prompt interrupt flag.
pub fn take_prompt_interrupt() -> bool {
    PROMPT_INTERRUPTED.swap(false, Ordering::SeqCst)
}

/// Guard that marks a prompt region as SIGINT-interruptible.
pub struct PromptInterruptGuard;

impl PromptInterruptGuard {
    /// Enter a prompt-interruptible region.
    #[must_use]
    pub fn new() -> Self {
        PROMPT_INTERRUPTED.store(false, Ordering::SeqCst);
        PROMPT_INTERRUPTIBLE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Default for PromptInterruptGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PromptInterruptGuard {
    fn drop(&mut self) {
        PROMPT_INTERRUPTIBLE.store(false, Ordering::SeqCst);
        PROMPT_INTERRUPTED.store(false, Ordering::SeqCst);
    }
}
