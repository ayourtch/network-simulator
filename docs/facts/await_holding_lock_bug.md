# Await Holding Lock Bug Fact

Holding a `MutexGuard` across an `await` point can cause deadlocks and is flagged by Clippy (`await_holding_lock`). The fix is to limit the lock’s scope, performing all RNG work before any async `await` and dropping the guard early.
