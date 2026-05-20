# Beta Testing Guide — openshell-signal-chain

> **⚠️ Alpha Software Notice**
> This crate is in active development. APIs may change. Test thoroughly before production use.

## Adding openshell-signal-chain to Your Rust Project

### Cargo.toml Dependency

```toml
[dependencies]
openshell-signal-chain = "0.1.0"
```

Or from a local path (for OpenShell contributors):

```toml
[dependencies]
openshell-signal-chain = { path = "../openshell-signal-chain" }
```

### Quick Start

```rust
use openshell_signal_chain::{Dial, Room, SignalChain};

fn main() {
    // Create a signal chain
    let mut chain = SignalChain::new("my-app");

    // Add a room
    let room = chain.room("data");
    room.add_snap(serde_json::json!({"key": "value"}), 1.0);
    room.add_inference(serde_json::json!({"hypothesis": "guess"}), 0.7);

    // Query at any dial level
    let results = room.query(Dial::new(0.5));
    println!("Results: {:?}", results);
}
```

## Using in an OpenShell Sandbox Context

The signal chain integrates with OpenShell's sandbox architecture for AI task routing:

```rust
use openshell_signal_chain::{Dial, Room, SignalChain, DIAL_FORMAL, DIAL_CREATIVE};

// Create chain matching sandbox context
let mut chain = SignalChain::new("sandbox-tasks");

// Formal room: algorithm-heavy tasks
let mut formal = chain.room_with_dial("formal", DIAL_FORMAL);
formal.add_snap(serde_json::json!({"task": "proof", "verified": true}), 1.0);

// Creative room: generative tasks
let mut creative = chain.room_with_dial("creative", DIAL_CREATIVE);
creative.add_inference(serde_json::json!({"task": "story", "genre": "scifi"}), 0.9);

// Route based on dial position
fn route_task(chain: &SignalChain, dial: Dial) -> Vec<serde_json::Value> {
    chain.query_all(dial).into_values().flatten().collect()
}
```

## What to Test

### 1. Dial Behavior

Test that dials correctly filter snaps vs inferences:

```rust
let mut room = Room::new("test");
room.add_snap(serde_json::json!({"snap": true}), 1.0);
room.add_inference(serde_json::json!({"inf": true}), 0.8);

// Hard dial: only snaps
assert_eq!(room.query(Dial::hard()).len(), 1);

// Soft dial: snaps + inferences
assert_eq!(room.query(Dial::soft()).len(), 2);

// Middle dial: threshold check
let mid_dial = Dial::new(0.3); // threshold = 0.7
// 0.8 >= 0.7, so inference included
assert_eq!(room.query(mid_dial).len(), 2);
```

### 2. Snap/Inference Queries

```rust
let mut room = Room::new("test");
room.add_snap(serde_json::json!({"a": 1}), 1.0);
room.add_snap(serde_json::json!({"b": 2}), 0.9);
room.add_inference(serde_json::json!({"c": 3}), 0.7);

// Query snaps only
let snaps = room.query_snaps();
assert_eq!(snaps.len(), 2);

// Query inferences at threshold
let infs = room.query_inferences(Dial::new(0.5)); // threshold = 0.5
assert_eq!(infs.len(), 1);
```

### 3. Cascade Behavior

```rust
let mut parent = Room::new("parent");
let mut child = Room::new("child");
parent.children.insert("child".to_string(), child);

parent.add_inference(serde_json::json!({"h": "test"}), 0.9); // > 0.5
parent.cascade(1);

let child_snaps = parent.children.get("child").unwrap().snaps.len();
assert!(child_snaps > 0);
```

### 4. SignalChain Operations

```rust
let mut chain = SignalChain::new("test");

// Get or create room
let room = chain.room("my-room");
assert_eq!(room.name, "my-room");

// Room with specific dial
let hard_room = chain.room_with_dial("hard", Dial::hard());
assert_eq!(hard_room.dialect.position, 0.0);

// Query all rooms
chain.room("a").add_snap(serde_json::json!({"a": 1}), 1.0);
chain.room("b").add_inference(serde_json::json!({"b": 2}), 0.9);

let all = chain.query_all(Dial::soft());
assert_eq!(all.len(), 2);
```

### 5. Serialization Round-Trip

```rust
use openshell_signal_chain::{Dial, Room, SignalChain};

let mut chain = SignalChain::new("test");
chain.room("r").add_snap(serde_json::json!({"x": 1}), 1.0);

// Serialize
let json = serde_json::to_string(&chain).unwrap();

// Deserialize
let restored: SignalChain = serde_json::from_str(&json).unwrap();
assert_eq!(restored.name, "test");
```

## How to Report Bugs

### Required Information

1. **Reproduction steps** — minimal code that shows the bug
2. **Expected behavior** — what you expected to happen
3. **Actual behavior** — what actually happened
4. **Rust/Cargo version:**
   ```bash
   rustc --version
   cargo --version
   ```
5. **Platform:**
   ```bash
   uname -a
   ```
6. **Cargo tree output:**
   ```bash
   cargo tree -p openshell-signal-chain
   ```
7. **Git revision:**
   ```bash
   git log --oneline -1
   ```

### Debug Output to Include

```bash
# Enable trace logging
RUST_LOG=trace cargo test -p openshell-signal-chain -- --nocapture 2>&1 | head -100

# List discovered tests
cargo test -p openshell-signal-chain -- --list
```

### Example Bug Report

```markdown
## Bug: Cascade doesn't propagate to nested children

### Steps to Reproduce
1. Create parent room with high-confidence inference
2. Add child room with grandchild
3. Call cascade(2)

### Expected
Grandchild receives snaps from cascade.

### Actual
Grandchild is empty.

### Code
```rust
let mut root = Room::new("root");
let mut child = Room::new("child");
let mut grandchild = Room::new("grandchild");
child.children.insert("grandchild".to_string(), grandchild);
root.children.insert("child".to_string(), child);
root.add_inference(serde_json::json!({"h": "test"}), 0.95);
root.cascade(2);
```

### Environment
- rustc 1.88.0
- cargo 1.88.0
- Linux aarch64
```

## Known Limitations

1. **Alpha API** — APIs may change between 0.1.x versions
2. **No persistence** — SignalChains are in-memory only; serialize to JSON for storage
3. **No network features** — Currently single-process only
4. **Tokio dependency** — Requires async runtime even for sync operations
5. **No authentication** — SignalChain has no access control
6. **Limited query language** — Only threshold-based filtering, no complex queries
7. **ARM64 tested** — Only verified on Oracle Cloud ARM64; other platforms untested

## Performance Expectations

- **Room creation:** < 1µs
- **Snap/inference add:** < 1µs
- **Query (100 items):** < 10µs
- **Cascade (depth 10, 10 items each):** < 100µs

These are approximate; profile for your workload.

## Getting Help

- GitHub Issues: https://github.com/NVIDIA/OpenShell/issues
- OpenShell Discord: https://discord.gg/OpenShell
