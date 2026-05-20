# Debugging Guide — openshell-signal-chain

## Running the Test Suite with Debug Output

### Basic Test Run
```bash
cd crates/openshell-signal-chain
cargo test -p openshell-signal-chain
```

### With All Output (including print statements)
```bash
cargo test -p openshell-signal-chain -- --nocapture
```

### Run a Single Test
```bash
cargo test -p openshell-signal-chain test_dial_bounds -- --nocapture
```

### Run Tests Matching a Pattern
```bash
cargo test -p openshell-signal-chain query -- --nocapture
```

### List All Available Tests
```bash
cargo test -p openshell-signal-chain -- --list
```

## Enabling Tracing/Logging

### Environment Variable Method
```bash
RUST_LOG=debug cargo test -p openshell-signal-chain -- --nocapture
RUST_LOG=trace cargo test -p openshell-signal-chain -- --nocapture
```

### Log Levels
- `error` — only errors
- `warn` — warnings and errors
- `info` — informational messages (default)
- `debug` — detailed debug info
- `trace` — very verbose, includes all function entry/exit

### Programmatic Tracing in Your Code

Add `tracing` to your source files:
```rust
use tracing::{debug, info, warn};

fn my_function() {
    debug!("Entering my_function with dial position");
    // ... logic
    info!("Query returned {} results", results.len());
}
```

### Using tracing-subscriber for Custom Output

If you want colored output or JSON logs:
```rust
tracing_subscriber::fmt()
    .with_env_filter("debug")
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

## Common Failure Modes and Diagnosis

### 1. "test timed out" / Hang

**Symptoms:** Tests hang indefinitely.

**Diagnosis:**
```bash
cargo test -p openshell-signal-chain -- --test-threads=1
```

If it hangs on a specific test, that test likely has a deadlock or infinite loop.

**Common causes in this crate:**
- `tokio::spawn` tasks that never complete (we use full features)
- Channels that never close

### 2. "assertion failed" — Confidence Out of Range

**Symptoms:**
```
assertion failed: confidence >= 0.0 && confidence <= 1.0
```

**Cause:** You're passing a confidence value outside 0.0-1.0 to `Snap::new()` or `Inference::new()`.

**Fix:** The crate auto-clamps values, but if you see this in tests, check your test values:
```rust
// Wrong:
room.add_snap(serde_json::json!({"x": 1}), 1.5);

// Correct:
room.add_snap(serde_json::json!({"x": 1}), 1.0);
```

### 3. "inference not returned at dial X"

**Symptoms:** An inference you added doesn't appear in query results.

**Explanation:** This is expected behavior. At dial position `p`, the inference threshold is `1.0 - p`. An inference with confidence `c` is only returned if `c >= 1.0 - p`.

**Diagnosis table:**
| Dial Position | Threshold | Inference (0.7 conf) Returned? |
|--------------|-----------|-------------------------------|
| 0.0 (hard)   | 1.0       | No                            |
| 0.3          | 0.7       | Yes (0.7 >= 0.7)              |
| 0.5          | 0.5       | Yes                           |
| 0.7          | 0.3       | Yes                           |
| 1.0 (soft)   | 0.0       | Yes                           |

### 4. Dial Position Clamping

**Symptoms:** `Dial::new(2.0)` returns position 1.0 instead of 2.0.

**This is by design.** Dials are clamped to [0.0, 1.0].

### 5. Cascade Does Nothing

**Symptoms:** `room.cascade(depth)` doesn't populate child rooms.

**Prerequisites for cascade to work:**
1. The room must have child rooms in `room.children`
2. The room must have inferences with confidence > 0.5
3. `depth > 0`

**Example:**
```rust
let mut parent = Room::new("parent");
let mut child = Room::new("child");
parent.children.insert("child".to_string(), child);
parent.add_inference(serde_json::json!({"hypothesis": "test"}), 0.7);
parent.cascade(1); // Now child has snaps
```

## ARM64 / Oracle Cloud Checks

### Verify You're on ARM64
```bash
uname -m
# Expected output: aarch64
```

### Build for Your Target
```bash
# Native build (default)
cargo build -p openshell-signal-chain

# Cross-compile check (validates without emitting code)
cargo check -p openshell-signal-chain --target aarch64-unknown-linux-gnu
```

### Known ARM64 Considerations
- All Rust stdlib code is ARM64-safe
- No platform-specific assembly in this crate
- tokio full features include epoll/kqueue — kqueue is used on ARM64 Linux

### Test on ARM64
```bash
# Run full test suite
cargo test -p openshell-signal-chain

# With output capture disabled
cargo test -p openshell-signal-chain -- --nocapture

# Run specific test module
cargo test -p openshell-signal-chain -- dial::tests --nocapture
```

## Profiling Tips

### Using cargo-flamegraph
```bash
cargo install flamegraph
cargo flamegraph -p openshell-signal-chain -- test_room_query_at_high_dial
```

### Using cargo-instruments (macOS)
```bash
cargo instruments -p openshell-signal-chain --example basic_dial
```

## Getting Help

If you're stuck:
1. Run with `RUST_LOG=trace` and capture output
2. Run `cargo test -- --list` to verify tests are discovered
3. Check GitHub issues at https://github.com/NVIDIA/OpenShell/issues
4. Include `cargo tree -p openshell-signal-chain` in bug reports
