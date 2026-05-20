// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Tagged query results that preserve the fact-vs-hypothesis distinction.
//!
//! The untagged [`Room::query`] method returns all results as `serde_json::Value`,
//! losing information about whether each value came from a snap or an inference.
//! [`QueryResult`] preserves that information so callers can handle facts and
//! hypotheses differently.
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::{Dial, Room, QueryResult};
//!
//! let mut room = Room::new("test");
//! room.add_snap(serde_json::json!({"fact": true}), 1.0);
//! room.add_inference(serde_json::json!({"maybe": true}), 0.7);
//!
//! let results = room.query_tagged(Dial::new(0.5));
//! let (snaps, inferences): (Vec<_>, Vec<_>) = results
//!     .into_iter()
//!     .partition(QueryResult::is_snap);
//!
//! assert_eq!(snaps.len(), 1);
//! assert_eq!(inferences.len(), 1);
//! ```

use serde::{Deserialize, Serialize};

/// A tagged query result that preserves whether the value is a hard fact or a soft hypothesis.
///
/// Use [`Room::query_tagged`] instead of [`Room::query`] when you need to distinguish
/// between snaps (ground truth) and inferences (hypotheses) in the result set.
///
/// # Variants
///
/// - [`QueryResult::Snap`]: Value from a hard-locked [`Snap`](crate::Snap).
/// - [`QueryResult::Inference`]: Value from a soft [`Inference`](crate::Inference)
///   that passed the dial threshold.
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::{Dial, Room, QueryResult};
///
/// let mut room = Room::new("test");
/// room.add_snap(serde_json::json!({"confirmed": true}), 1.0);
/// room.add_inference(serde_json::json!({"possible": true}), 0.8);
///
/// let tagged = room.query_tagged(Dial::new(0.5));
/// assert!(tagged[0].is_snap());
/// assert!(tagged[1].is_inference());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResult {
    /// A hard-locked fact from a [`Snap`](crate::Snap).
    Snap(serde_json::Value),
    /// A soft hypothesis from an [`Inference`](crate::Inference) that passed the dial threshold.
    Inference(serde_json::Value),
}

impl QueryResult {
    /// Returns the inner JSON value regardless of variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::QueryResult;
    ///
    /// let r = QueryResult::Snap(serde_json::json!({"x": 1}));
    /// assert_eq!(r.value(), &serde_json::json!({"x": 1}));
    /// ```
    pub fn value(&self) -> &serde_json::Value {
        match self {
            Self::Snap(v) | Self::Inference(v) => v,
        }
    }

    /// Returns `true` if this result originated from a snap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::QueryResult;
    ///
    /// assert!(QueryResult::Snap(serde_json::json!({})).is_snap());
    /// assert!(!QueryResult::Inference(serde_json::json!({})).is_snap());
    /// ```
    pub fn is_snap(&self) -> bool {
        matches!(self, Self::Snap(_))
    }

    /// Returns `true` if this result originated from an inference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::QueryResult;
    ///
    /// assert!(QueryResult::Inference(serde_json::json!({})).is_inference());
    /// assert!(!QueryResult::Snap(serde_json::json!({})).is_inference());
    /// ```
    pub fn is_inference(&self) -> bool {
        matches!(self, Self::Inference(_))
    }
}
