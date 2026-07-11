// SPDX-License-Identifier: MIT
//! Watch mode — monitor the current folder/repo and sweep on change.
//!
//! Implements #26. A filesystem watcher ([`notify`]) reports change events; a
//! trailing-debounce window collapses a burst of saves into a single settled
//! signal, so rapid edits don't fan out into concurrent sweeps. The debounce
//! math ([`coalesce`]) and the ignore filter ([`is_ignored`]) are pure and
//! unit-tested; the generic [`run`] driver is exercised with a fake
//! [`ChangeStream`] so the loop is testable without a real watcher or provider.

use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

/// Default debounce window: a burst of saves within this span is one sweep.
pub const DEFAULT_DEBOUNCE_MS: u64 = 400;

/// Directory names whose changes must never trigger a sweep. `.talicode/` is
/// critical: the usage ledger is written *during* a sweep, so watching it would
/// feed back into an endless sweep loop.
const IGNORED_DIRS: [&str; 4] = [".git", "target", ".talicode", "node_modules"];

/// Whether a changed path lives under an ignored directory (so it's skipped).
pub fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|name| IGNORED_DIRS.contains(&name))
    })
}

/// Number of sweeps a sequence of change timestamps (ascending, in ms) collapses
/// into under a trailing-debounce `window`. A gap of at least `window_ms`
/// between consecutive events settles the current burst (one sweep); the final
/// burst always settles. Empty input ⇒ no sweeps.
pub fn coalesce(event_ms: &[u64], window_ms: u64) -> usize {
    let mut fires = 0usize;
    let mut prev: Option<u64> = None;
    for &t in event_ms {
        if let Some(p) = prev {
            if t.saturating_sub(p) >= window_ms {
                fires += 1; // the previous burst settled and fired
            }
        }
        prev = Some(t);
    }
    if prev.is_some() {
        fires += 1; // the final burst always settles
    }
    fires
}

/// A source of debounced change signals. Each [`next`](ChangeStream::next)
/// resolves once a settled burst of file changes has occurred, or `None` when
/// the source is exhausted (watcher dropped). The trait is the test seam: unit
/// tests drive [`run`] with an in-memory fake instead of a real watcher.
#[async_trait]
pub trait ChangeStream {
    /// Await the next settled change burst, or `None` when the stream ends.
    async fn next(&mut self) -> Option<()>;
}

/// Drive the watch loop: run `on_change` once per settled burst from `stream`.
/// Returns the number of sweeps run (meaningful in tests; the CLI runs until
/// `Ctrl-C`). Generic over the stream and the sweep future so tests supply a
/// fake source and a counting closure — no `notify`, no provider.
pub async fn run<S, F, Fut>(mut stream: S, mut on_change: F) -> usize
where
    S: ChangeStream,
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut sweeps = 0usize;
    while stream.next().await.is_some() {
        on_change().await;
        sweeps += 1;
    }
    sweeps
}

/// A [`ChangeStream`] backed by a real [`notify`] filesystem watcher.
///
/// A dedicated OS thread owns the watcher and applies the trailing debounce with
/// blocking receives (mirroring [`coalesce`]), forwarding one signal per settled
/// burst onto an async channel that [`ChangeStream::next`] awaits. The watcher is
/// held alive by this struct; dropping it disconnects the channel and stops the
/// thread. Changes under [`IGNORED_DIRS`] are filtered out at the source.
pub struct FsChangeStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    _watcher: notify::RecommendedWatcher,
}

impl FsChangeStream {
    /// Start watching `root` recursively with the given debounce `window`.
    pub fn start(root: &Path, window: Duration) -> notify::Result<Self> {
        use notify::{RecursiveMode, Watcher};

        let (raw_tx, raw_rx) = std::sync::mpsc::channel::<()>();
        let handler = move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if event.paths.iter().any(|p| !is_ignored(p)) {
                    let _ = raw_tx.send(());
                }
            }
        };
        let mut watcher = notify::recommended_watcher(handler)?;
        watcher.watch(root, RecursiveMode::Recursive)?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        std::thread::spawn(move || debounce_loop(raw_rx, window, tx));

        Ok(FsChangeStream {
            rx,
            _watcher: watcher,
        })
    }
}

/// Trailing-debounce loop for the watcher thread: block for the first event,
/// keep draining while events keep arriving within `window`, then emit one
/// settled signal. Ends when either channel disconnects.
fn debounce_loop(
    raw_rx: std::sync::mpsc::Receiver<()>,
    window: Duration,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    use std::sync::mpsc::RecvTimeoutError;
    while raw_rx.recv().is_ok() {
        // Coalesce the burst: swallow events until a `window`-long quiet gap.
        loop {
            match raw_rx.recv_timeout(window) {
                Ok(()) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if tx.send(()).is_err() {
            return; // async side gone
        }
    }
}

#[async_trait]
impl ChangeStream for FsChangeStream {
    async fn next(&mut self) -> Option<()> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn coalesce_collapses_a_burst_into_one_sweep() {
        // Ten saves 50ms apart, all within the 400ms window ⇒ one sweep.
        let burst: Vec<u64> = (0..10).map(|i| i * 50).collect();
        assert_eq!(coalesce(&burst, 400), 1);
    }

    #[test]
    fn coalesce_separates_bursts_by_a_quiet_gap() {
        // Two bursts separated by a 5s gap ⇒ two sweeps.
        assert_eq!(coalesce(&[0, 100, 5000, 5100], 400), 2);
    }

    #[test]
    fn coalesce_edge_cases() {
        assert_eq!(coalesce(&[], 400), 0);
        assert_eq!(coalesce(&[42], 400), 1);
        // Exactly `window` apart settles the first burst.
        assert_eq!(coalesce(&[0, 400], 400), 2);
    }

    #[test]
    fn ignored_dirs_never_trigger() {
        assert!(is_ignored(&PathBuf::from("/repo/.git/index")));
        assert!(is_ignored(&PathBuf::from("/repo/.talicode/usage.jsonl")));
        assert!(is_ignored(&PathBuf::from("/repo/target/debug/tali")));
        assert!(!is_ignored(&PathBuf::from("/repo/src/main.rs")));
    }

    /// Fake stream yielding a fixed number of settled bursts, then ending.
    struct FakeStream {
        remaining: usize,
    }

    #[async_trait]
    impl ChangeStream for FakeStream {
        async fn next(&mut self) -> Option<()> {
            if self.remaining == 0 {
                return None;
            }
            self.remaining -= 1;
            Some(())
        }
    }

    #[tokio::test]
    async fn run_sweeps_once_per_settled_burst() {
        let sweeps = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&sweeps);
        let ran = run(FakeStream { remaining: 3 }, || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;
        assert_eq!(ran, 3);
        assert_eq!(sweeps.load(Ordering::SeqCst), 3);
    }
}
