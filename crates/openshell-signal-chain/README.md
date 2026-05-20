# openshell-signal-chain

A Rust crate implementing the **Signal Chain Thesis**: every intelligent system needs a dial between hard-snapped algorithms and soft-inferenced models.

## The Dial Concept

The core idea is simple. Every query has a **dial** — a continuous slider from 0.0 to 1.0:

- **0.0 (Hard)**: Give me only verified, deterministic facts. No guesses.
- **0.5 (Balanced)**: Facts plus confident hypotheses.
- **1.0 (Soft)**: Everything — facts, guesses, wild speculation, creative output.

The dial controls a **threshold**: an inference only appears in results if its confidence ≥ `1.0 - dial_position`. At dial 0.0, the threshold is 1.0 (only absolute certainty). At dial 1.0, the threshold is 0.0 (everything passes).

## Quick Start

```rust
use openshell_signal_chain::{Dial, Room, SignalChain};

// Create a chain
let mut chain = SignalChain::new("my-ops");

// Add a room with sensor data
let sensors = chain.room("sonar");
sensors.add_snap(serde_json::json!({"depth": 87.2}), 1.0);        // hard fact
sensors.add_inference(serde_json::json!({"wreck": true}), 0.7);   // hypothesis

// Query at different dial levels
let hard = sensors.query(Dial::hard());   // [depth] — snap only
let soft = sensors.query(Dial::soft());   // [depth, wreck] — everything
let mid  = sensors.query(Dial::new(0.5)); // [depth, wreck] — threshold 0.5, 0.7 passes
```

## Core Types

| Type | What | Key Idea |
|------|------|----------|
| `Dial` | 0.0–1.0 slider | Controls snap vs. inference ratio |
| `Snap` | Hard-locked fact | Always appears in queries |
| `Inference` | Soft hypothesis | Filtered by dial threshold |
| `Room` | Fact-space | Contains snaps, inferences, children |
| `SignalChain` | Room collection | Global dial, cascade, query_all |

## Cascade

Rooms can have child rooms. The `cascade` method propagates the top inferences (sorted by confidence, descending) into children as snaps with a 0.8× confidence decay:

```rust
let mut parent = Room::new("fleet-hq");
parent.add_inference(serde_json::json!({"alert": "anomaly"}), 0.9);
parent.children.insert("drone-1".to_string(), Room::new("drone-1"));

parent.cascade(1);
// drone-1 now has a snap with confidence 0.72 (0.9 × 0.8)
```

## Preset Dials

```rust
use openshell_signal_chain::{DIAL_FORMAL, DIAL_ANALYSIS, DIAL_CREATIVE};

DIAL_FORMAL    // 0.0  — theorem provers, ISA semantics
DIAL_BATHY     // 0.1  — sonar readings, depth facts
DIAL_COMMIT    // 0.05 — git history, build logs
DIAL_ANALYSIS  // 0.4  — reasoning with snap anchors
DIAL_REVIEW    // 0.5  — equal weight
DIAL_EXTRAPOLATE // 0.7 — hypothesis generation
DIAL_CREATIVE  // 0.9  — story generation
DIAL_EXPLORATORY // 1.0 — pure inference
```

## Error Handling

```rust
use openshell_signal_chain::{Dial, SignalChainError};

// Clamped (silent)
let d = Dial::new(-0.5);  // position = 0.0

// Strict (returns error)
let result = Dial::try_new(-0.5);
assert!(matches!(result, Err(SignalChainError::InvalidDial(_))));
```

## How It Connects to OpenShell

This crate is part of the [OpenShell](https://github.com/SuperInstance/OpenShell) project — building intelligent systems that balance deterministic computation with probabilistic reasoning. The signal chain is the backbone of the PLATO room protocol, where tiles flow between rooms with confidence-aware routing.

## License

Apache-2.0
