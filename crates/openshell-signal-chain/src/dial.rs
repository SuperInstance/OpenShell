// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Dial — continuous control from hard-snapped to soft-inferenced.
//!
//! The dial is the central abstraction of the Signal Chain Thesis. It controls
//! the ratio of hard (deterministic, provable) vs. soft (probabilistic, generative)
//! reasoning in every query.
//!
//! # Position Semantics
//!
//! - **0.0** — Hard algorithm: theorem provers, ISA semantics, certified traces
//! - **0.5** — Balanced: equal weight to algorithm and inference
//! - **1.0** — Pure inference: story generation, creative fill, exploration
//!
//! # Threshold Mechanics
//!
//! The inference threshold is `1.0 - position`. An inference passes when its
//! confidence >= threshold.
//!
//! | Position | Threshold | Meaning |
//! |----------|-----------|---------|
//! | 0.0      | 1.0       | Only absolute-certainty inferences |
//! | 0.5      | 0.5       | Confident inferences pass |
//! | 1.0      | 0.0       | All inferences pass |
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::Dial;
//!
//! // Create a dial at the balanced position
//! let d = Dial::new(0.5);
//! assert_eq!(d.snap_weight(), 0.5);
//! assert_eq!(d.inference_weight(), 0.5);
//!
//! // Validate strictly — rejects out-of-range values
//! let strict = Dial::try_new(1.5);
//! assert!(strict.is_err());
//! ```

use serde::{Deserialize, Serialize};

use crate::SignalChainError;

/// A dial controlling the ratio of hard-snapped vs. soft-inferenced reasoning.
///
/// Position 0.0 = deterministic, provable, certifiable (theorem provers, FLUX ISA, H1 cohomology).
/// Position 1.0 = probabilistic, generative, exploratory (story generation, creative fill).
///
/// Use [`Dial::new`] for clamped creation (silently clamps to range) or
/// [`Dial::try_new`] for strict validation (returns error on out-of-range).
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::Dial;
///
/// let d = Dial::new(0.7);
/// assert_eq!(d.position, 0.7);
///
/// // Clamped: -0.5 becomes 0.0
/// let clamped = Dial::new(-0.5);
/// assert_eq!(clamped.position, 0.0);
///
/// // Strict: rejects invalid values
/// let strict = Dial::try_new(-0.5);
/// assert!(strict.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dial {
    /// Position on the dial: 0.0 (hard) to 1.0 (soft).
    ///
    /// Always in range [0.0, 1.0] when constructed via [`Dial::new`] or [`Dial::try_new`].
    pub position: f64,
}

impl Dial {
    /// Create a new dial with given position, clamped to [0.0, 1.0].
    ///
    /// Values below 0.0 are clamped to 0.0; values above 1.0 are clamped to 1.0.
    /// For strict validation that returns an error, use [`Dial::try_new`].
    ///
    /// # Parameters
    ///
    /// - `position`: Desired dial position. Clamped to [0.0, 1.0].
    ///
    /// # Returns
    ///
    /// A [`Dial`] with the (possibly clamped) position.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::new(0.5);
    /// assert_eq!(d.position, 0.5);
    ///
    /// let clamped_low = Dial::new(-1.0);
    /// assert_eq!(clamped_low.position, 0.0);
    ///
    /// let clamped_high = Dial::new(2.0);
    /// assert_eq!(clamped_high.position, 1.0);
    /// ```
    pub fn new(position: f64) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
        }
    }

    /// Create a new dial with strict validation.
    ///
    /// Unlike [`Dial::new`], this returns an error if the position is outside
    /// the valid [0.0, 1.0] range rather than clamping.
    ///
    /// # Parameters
    ///
    /// - `position`: Desired dial position. Must be in [0.0, 1.0].
    ///
    /// # Returns
    ///
    /// - `Ok(Dial)` if position is in range.
    /// - `Err(`[`SignalChainError::InvalidDial`]`)` if position is out of range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, SignalChainError};
    ///
    /// let valid = Dial::try_new(0.5).unwrap();
    /// assert_eq!(valid.position, 0.5);
    ///
    /// let invalid = Dial::try_new(-0.5);
    /// assert!(matches!(invalid, Err(SignalChainError::InvalidDial(_))));
    /// ```
    pub fn try_new(position: f64) -> Result<Self, SignalChainError> {
        if (0.0..=1.0).contains(&position) {
            Ok(Self { position })
        } else {
            Err(SignalChainError::InvalidDial(position))
        }
    }

    /// Create a dial at 0.0 — pure hard algorithm mode.
    ///
    /// No inferences pass unless they have absolute confidence (1.0).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::hard();
    /// assert_eq!(d.position, 0.0);
    /// assert_eq!(d.snap_weight(), 1.0);
    /// ```
    pub fn hard() -> Self {
        Self { position: 0.0 }
    }

    /// Create a dial at 1.0 — pure soft inference mode.
    ///
    /// All inferences pass regardless of confidence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::soft();
    /// assert_eq!(d.position, 1.0);
    /// assert_eq!(d.inference_weight(), 1.0);
    /// ```
    pub fn soft() -> Self {
        Self { position: 1.0 }
    }

    /// Weight for hard-snapped facts (inverse of position).
    ///
    /// Returns `1.0 - position`. At position 0.0, snap weight is 1.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::new(0.3);
    /// assert_eq!(d.snap_weight(), 0.7);
    /// ```
    pub fn snap_weight(&self) -> f64 {
        1.0 - self.position
    }

    /// Weight for soft inferences (equals position).
    ///
    /// Returns `position`. At position 1.0, inference weight is 1.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::new(0.3);
    /// assert_eq!(d.inference_weight(), 0.3);
    /// ```
    pub fn inference_weight(&self) -> f64 {
        self.position
    }

    /// Threshold for including inferences (`1.0 - position`).
    ///
    /// At 0.0: threshold = 1.0 (only absolute inferences).
    /// At 1.0: threshold = 0.0 (all inferences).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::new(0.3);
    /// assert_eq!(d.inference_threshold(), 0.7);
    /// ```
    pub fn inference_threshold(&self) -> f64 {
        1.0 - self.position
    }

    /// Check if an inference passes the threshold for this dial.
    ///
    /// An inference passes when `confidence >= inference_threshold()`.
    ///
    /// # Parameters
    ///
    /// - `confidence`: The confidence value of the inference to check.
    ///
    /// # Returns
    ///
    /// `true` if the inference should be included at this dial level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Dial;
    ///
    /// let d = Dial::new(0.5); // threshold = 0.5
    /// assert!(d.accepts_inference(0.7));
    /// assert!(!d.accepts_inference(0.3));
    /// ```
    pub fn accepts_inference(&self, confidence: f64) -> bool {
        confidence >= self.inference_threshold()
    }
}

impl Default for Dial {
    /// Default dial at the balanced midpoint (0.5).
    fn default() -> Self {
        Self { position: 0.5 }
    }
}

// --- Preset dials for common use cases ---

/// Pure formal reasoning — theorem provers, FLUX ISA, H1 cohomology.
///
/// Only snaps and absolute-certainty inferences (confidence = 1.0) pass.
pub const DIAL_FORMAL: Dial = Dial { position: 0.0 };

/// Hard bathydata — sonar readings, coordinates, depth facts.
///
/// Near-hard: allows only very high confidence inferences.
pub const DIAL_BATHY: Dial = Dial { position: 0.1 };

/// Git history, build logs, certifiable traces.
///
/// Almost hard: trusts build system outputs and verified commits.
pub const DIAL_COMMIT: Dial = Dial { position: 0.05 };

/// Reasoning with snaps as anchors.
///
/// Balanced toward hard facts but allows moderate inferences.
pub const DIAL_ANALYSIS: Dial = Dial { position: 0.4 };

/// Equal weight to algorithm and inference.
///
/// The balanced midpoint — snaps always, inferences at confidence >= 0.5.
pub const DIAL_REVIEW: Dial = Dial { position: 0.5 };

/// Hypothesis generation, creative fill.
///
/// Favors inferences — threshold is low (0.3).
pub const DIAL_EXTRAPOLATE: Dial = Dial { position: 0.7 };

/// Story generation, game narrative.
///
/// Very soft — almost everything passes.
pub const DIAL_CREATIVE: Dial = Dial { position: 0.9 };

/// Pure inference, no snap constraints.
///
/// Everything passes — threshold is 0.0.
pub const DIAL_EXPLORATORY: Dial = Dial { position: 1.0 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dial_position() {
        let d = Dial::new(0.5);
        assert_eq!(d.position, 0.5);
        assert_eq!(d.snap_weight(), 0.5);
        assert_eq!(d.inference_weight(), 0.5);
    }

    #[test]
    fn test_dial_bounds_clamp() {
        let d = Dial::new(-0.5);
        assert_eq!(d.position, 0.0);

        let d = Dial::new(1.5);
        assert_eq!(d.position, 1.0);
    }

    #[test]
    fn test_try_new_valid() {
        assert!(Dial::try_new(0.0).is_ok());
        assert!(Dial::try_new(0.5).is_ok());
        assert!(Dial::try_new(1.0).is_ok());
    }

    #[test]
    fn test_try_new_invalid() {
        assert!(Dial::try_new(-0.1).is_err());
        assert!(Dial::try_new(1.1).is_err());
        assert!(Dial::try_new(f64::NAN).is_err());
        assert!(Dial::try_new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_dial_boundary_values() {
        let zero = Dial::new(0.0);
        assert_eq!(zero.position, 0.0);
        assert_eq!(zero.snap_weight(), 1.0);
        assert_eq!(zero.inference_weight(), 0.0);

        let one = Dial::new(1.0);
        assert_eq!(one.position, 1.0);
        assert_eq!(one.snap_weight(), 0.0);
        assert_eq!(one.inference_weight(), 1.0);
    }

    #[test]
    fn test_threshold() {
        let d = Dial::new(0.0); // hard - threshold = 1.0
        assert!(!d.accepts_inference(0.9));
        assert!(d.accepts_inference(1.0));

        let d = Dial::new(0.5); // threshold = 0.5
        assert!(!d.accepts_inference(0.4));
        assert!(d.accepts_inference(0.5));
        assert!(d.accepts_inference(0.9));

        let d = Dial::new(1.0); // soft - threshold = 0.0
        assert!(d.accepts_inference(0.0));
        assert!(d.accepts_inference(0.1));
    }
}
