# openshell-signal-chain

A Rust crate implementing the **Signal Chain Thesis**: a structured way to mix
hard-locked facts with soft hypotheses by turning a single dial.

---

## What it is

Most intelligent systems sit somewhere between two extremes:

- **Deterministic algorithms** — theorem provers, sensor readings, certified
  build logs. Results are binary: a fact is true or it isn't.
- **Probabilistic models** — language models, classifiers, pattern matchers.
  Results are distributions: a claim is likely, unlikely, or somewhere in
  between.

This crate gives you a data structure that holds both at once and lets you
*query* them at whatever point on that spectrum you need. A **Dial** at `0.0`
gives you only hard facts. A Dial at `1.0` gives you everything, including
wild guesses. Everything in between is a controlled mix.

The core types are:

| Type | Role |
|------|------|
| `Dial` | A `f64` in [0.0, 1.0] that sets the hard-to-soft ratio |
| `Snap` | A hard-locked fact, always returned by queries |
| `Inference` | A soft hypothesis, returned only if its confidence clears the dial threshold |
| `Room` | A named fact-space that holds snaps and inferences |
| `SignalChain` | A named collection of rooms with a shared global dial |

---

## Quick start

Add the dependency:

```toml
[dependencies]
openshell-signal-chain = "0.1"
serde_json = "1"
```

Then:

```rust
use openshell_signal_chain::{Dial, Room, SignalChain};

// Create a chain for your domain
let mut chain = SignalChain::new("fleet-ops");

// Add a room. Rooms hold snaps (facts) and inferences (hypotheses).
let sonar = chain.room("sonar-array");
sonar.add_snap(serde_json::json!({"depth_m": 87.2, "bearing": 127.4}), 1.0);
sonar.add_inference(serde_json::json!({"classification": "wreckage"}), 0.7);
sonar.add_inference(serde_json::json!({"classification": "rock"}), 0.3);

// Hard query: dial at 0.0, threshold = 1.0 — only snaps pass
let hard = sonar.query(Dial::hard());
assert_eq!(hard.len(), 1); // just the depth/bearing fact

// Balanced query: dial at 0.5, threshold = 0.5 — the 0.7 inference passes
let mid = sonar.query(Dial::new(0.5));
assert_eq!(mid.len(), 2); // snap + "wreckage" (0.7 >= 0.5)

// Soft query: dial at 1.0, threshold = 0.0 — everything passes
let soft = sonar.query(Dial::soft());
assert_eq!(soft.len(), 3); // snap + both inferences
```

---

## How the dial works

The dial is a `f64` in `[0.0, 1.0]`. Its only job is to compute an
**inference threshold**:

```
threshold = 1.0 - dial_position
```

An inference is included in a query result when:

```
inference.confidence >= threshold
```

Snaps are **always** included, regardless of dial position.

### Worked example

Say a room has:

- 1 snap: `depth_m = 87.2` (confidence 1.0)
- 3 inferences: confidences 0.9, 0.6, 0.2

| Dial | Threshold | Snap | 0.9 | 0.6 | 0.2 | Total |
|------|-----------|------|-----|-----|-----|-------|
| 0.0  | 1.0       | ✓    | ✗   | ✗   | ✗   | 1     |
| 0.2  | 0.8       | ✓    | ✓   | ✗   | ✗   | 2     |
| 0.5  | 0.5       | ✓    | ✓   | ✓   | ✗   | 3     |
| 0.8  | 0.2       | ✓    | ✓   | ✓   | ✓   | 4     |
| 1.0  | 0.0       | ✓    | ✓   | ✓   | ✓   | 4     |

### Creating a dial

```rust
use openshell_signal_chain::{Dial, SignalChainError};

// Clamped construction — silently clamps out-of-range values
let d = Dial::new(0.5);
let clamped = Dial::new(-0.5); // position = 0.0, no error

// Strict construction — returns Err if out of range
let ok = Dial::try_new(0.5);                   // Ok(Dial { position: 0.5 })
let err = Dial::try_new(1.5);                  // Err(SignalChainError::InvalidDial(1.5))

// Convenience constructors
let hard = Dial::hard(); // 0.0
let soft = Dial::soft(); // 1.0
```

### Derived values

```rust
let d = Dial::new(0.3);

d.snap_weight()         // 0.7 — how much weight hard facts carry
d.inference_weight()    // 0.3 — how much weight soft hypotheses carry
d.inference_threshold() // 0.7 — minimum confidence for an inference to pass
d.accepts_inference(0.8) // true  (0.8 >= 0.7)
d.accepts_inference(0.5) // false (0.5 < 0.7)
```

### Preset dials

These are `const` values for common use cases — import and use directly:

```rust
use openshell_signal_chain::{
    DIAL_FORMAL,      // 0.0  — theorem provers, ISA semantics, certified proofs
    DIAL_COMMIT,      // 0.05 — git history, build logs, verified traces
    DIAL_BATHY,       // 0.1  — sonar readings, depth facts, hard sensor data
    DIAL_ANALYSIS,    // 0.4  — reasoning anchored on snaps, moderate inferences
    DIAL_REVIEW,      // 0.5  — equal weight, snaps + inferences >= 0.5
    DIAL_EXTRAPOLATE, // 0.7  — hypothesis generation, threshold 0.3
    DIAL_CREATIVE,    // 0.9  — story generation, almost everything passes
    DIAL_EXPLORATORY, // 1.0  — pure inference, threshold 0.0
};
```

---

## API overview

### `Snap`

A hard-locked fact. Always returned by queries, regardless of dial position.
Stores a `serde_json::Value`, a confidence, the room's dial at creation time,
and a UTC timestamp.

```rust
use openshell_signal_chain::Snap;

// Explicit confidence
let s = Snap::new(serde_json::json!({"temp": 72.3}), 0.95, 0.0);

// Shorthand for confidence = 1.0
let absolute = Snap::absolute(serde_json::json!({"theorem": "proved"}), 0.0);
```

### `Inference`

A soft hypothesis. Filtered by the dial threshold during queries.
Same fields as `Snap` (hypothesis JSON, confidence, dial position, timestamp).

```rust
use openshell_signal_chain::Inference;

// Explicit confidence
let inf = Inference::new(serde_json::json!({"target": "vessel"}), 0.7, 0.5);

// Shorthand constructors
let likely     = Inference::likely(serde_json::json!({"anomaly": true}), 0.3);      // confidence = 0.8
let speculative = Inference::speculative(serde_json::json!({"maybe": true}), 0.5);  // confidence = 0.5
```

### `Room`

A named fact-space. Holds snaps, inferences, child rooms (for hierarchies),
and arbitrary metadata.

```rust
use openshell_signal_chain::{Dial, Room, QueryResult};

// Construction
let mut room = Room::new("sonar");                        // default dial 0.5
let mut hard_room = Room::with_dial("proofs", Dial::hard()); // dial 0.0

// Adding facts
room.add_snap(serde_json::json!({"depth": 42.5}), 1.0);   // returns &mut Snap
room.add_absolute(serde_json::json!({"confirmed": true})); // confidence = 1.0 shorthand
room.add_inference(serde_json::json!({"possible": "X"}), 0.7); // returns &mut Inference

// Querying
let all: Vec<serde_json::Value> = room.query(Dial::new(0.5));
let snaps_only: Vec<serde_json::Value> = room.query_snaps();
let infs_only: Vec<serde_json::Value> = room.query_inferences(Dial::new(0.5));

// Tagged query — preserves snap vs. inference distinction
let tagged: Vec<QueryResult> = room.query_tagged(Dial::new(0.5));
for item in &tagged {
    match item {
        QueryResult::Snap(v)      => println!("fact: {v}"),
        QueryResult::Inference(v) => println!("hypothesis: {v}"),
    }
}
```

### `SignalChain`

A named collection of rooms with a global dial. Rooms can override it locally.

```rust
use openshell_signal_chain::{Dial, SignalChain, DIAL_ANALYSIS};

// Construction
let mut chain = SignalChain::new("ops");                   // global dial 0.5
let mut strict = SignalChain::with_dial("formal", Dial::hard()); // global dial 0.0

// Room management
let room = chain.room("sensors");               // get-or-create, inherits global dial
let hard = chain.room_with_dial("proofs", Dial::hard()); // override dial for this room
let exists: Option<&Room>     = chain.get_room("sensors");
let mut_ref: Option<&mut Room> = chain.get_room_mut("sensors");
let validated = chain.create_room("checked");   // returns Err if name is empty

// Traversal — visit rooms in a specific order, skipping missing ones
let rooms = chain.traverse(&["sensors", "analysis", "nonexistent"]);
// returns Vec<&Room> with 2 elements ("nonexistent" is skipped)

// Cascade — propagate inferences from one room as snaps into all sibling rooms
chain.cascade_from("analysis", 1);

// Query all rooms at once
let all: HashMap<String, Vec<serde_json::Value>> = chain.query_all(Dial::new(0.5));
```

---

## Cascade explained

Cascade is how information flows between rooms. There are two variants:

**`Room::cascade(depth)`** — propagates from a room down to its *child rooms*
(the `children: HashMap<String, Room>` field).

**`SignalChain::cascade_from(origin, depth)`** — propagates from one named room
to all *sibling rooms* in the chain.

Both follow the same rules:

1. Sort the origin room's inferences by confidence, descending.
2. Keep only those with confidence > 0.5.
3. Take the top 2.
4. For each destination room, add each selected inference as a **snap** with
   confidence multiplied by **0.8** (the decay factor).
5. The snap also gets a `_source` field injected with the original confidence,
   so downstream code can see where it came from.

The decay means that each hop reduces confidence: a 0.9 inference becomes a
0.72 snap. This prevents speculative hypotheses from accumulating into false
certainty across many cascade levels.

### Worked example

```rust
use openshell_signal_chain::{Room, SignalChain};

let mut chain = SignalChain::new("intel");

// Origin: three inferences at varying confidence
let hq = chain.room("hq");
hq.add_inference(serde_json::json!({"alert": "anomaly-A"}), 0.9); // selected (rank 1)
hq.add_inference(serde_json::json!({"alert": "anomaly-B"}), 0.7); // selected (rank 2)
hq.add_inference(serde_json::json!({"alert": "noise"}), 0.3);     // dropped (< 0.5)

// Sibling room, currently empty
chain.room("field-unit");

// Cascade: hq → all siblings
chain.cascade_from("hq", 1);

let field = chain.get_room("field-unit").unwrap();
assert_eq!(field.snaps.len(), 2);
assert_eq!(field.snaps[0].confidence, 0.9 * 0.8); // 0.72
assert_eq!(field.snaps[1].confidence, 0.7 * 0.8); // 0.56
```

After cascade, `field-unit` has two snaps. They will always appear in queries
because snaps bypass the dial threshold. The 0.3-confidence noise was never
promoted.

### Why only the top 2?

Cascade is intentionally conservative. Flooding every child with every
hypothesis would collapse the distinction between facts and guesses. By
promoting only the two most confident inferences, cascade acts like a
*confidence gate*: information that hasn't earned sufficient confidence stays
soft; only the strongest signals harden.

### Why 0.8× decay?

Each hop represents a degree of indirection. A sensor reading is more
trustworthy than an inference derived from it, which is more trustworthy than
an inference derived from *that* inference. The 0.8× factor encodes this
intuition — a chain of three hops reduces confidence to roughly half
(`0.8³ ≈ 0.51`).

---

## Error handling

```rust
use openshell_signal_chain::{Dial, SignalChain, SignalChainError};

// InvalidDial: position outside [0.0, 1.0]
match Dial::try_new(1.5) {
    Err(SignalChainError::InvalidDial(v)) => println!("bad dial: {v}"),
    _ => {}
}

// EmptyName: room name must not be empty
match chain.create_room("") {
    Err(SignalChainError::EmptyName) => println!("name required"),
    _ => {}
}
```

When you don't need the error, `Dial::new` silently clamps:

```rust
let d = Dial::new(-99.0); // position = 0.0
let d = Dial::new(99.0);  // position = 1.0
```

---

## Serialization

All types derive `serde::Serialize` and `serde::Deserialize`. A `Room` round-trips
through JSON cleanly, including its `children` and `metadata` maps.

```rust
let json = serde_json::to_string(&room).unwrap();
let restored: Room = serde_json::from_str(&json).unwrap();
```

---

## Examples

The `examples/` directory has runnable programs:

```
cargo run --example basic_dial         # Dial mechanics
cargo run --example signal_chain_room  # Room queries at different dial levels
cargo run --example inference_routing  # Cascade between rooms
cargo run --example fleet_operations   # Full fleet scenario
```

---

## License

Apache-2.0. Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES.
