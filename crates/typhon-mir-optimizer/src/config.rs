//! Optimization configuration.

use bitflags::bitflags;

bitflags! {
    /// Flags for enabling/disabling individual optimization passes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PassFlags: u8 {
        /// Enable constant folding pass.
        const CONSTANT_FOLDING = 1 << 0;
        /// Enable dead code elimination pass.
        const DEAD_CODE = 1 << 1;
        /// Enable escape analysis pass.
        const ESCAPE_ANALYSIS = 1 << 2;
        /// Always inline functions called only once.
        const INLINE_SINGLE_USE = 1 << 3;
        /// Enable function inlining pass.
        const INLINING = 1 << 4;
        /// Enable reference count optimization pass.
        const REFCOUNT_OPT = 1 << 5;
    }
}

/// Configuration for the MIR optimizer.
#[derive(Clone, Copy, Debug)]
pub struct OptimizerConfig {
    /// Optimization level (0-3).
    pub level: u8,
    /// Maximum iterations for iterative passes.
    pub max_iterations: usize,
    /// Maximum function size to inline (default: 20 instructions).
    pub max_inline_size: usize,
    /// Cost/benefit threshold for inlining (default: 10).
    pub inline_threshold: usize,
    /// Enabled optimization passes.
    pub passes: PassFlags,
}

impl OptimizerConfig {
    /// Check if constant folding is enabled.
    #[must_use]
    pub const fn constant_folding_enabled(&self) -> bool {
        self.passes.contains(PassFlags::CONSTANT_FOLDING)
    }

    /// Check if dead code elimination is enabled.
    #[must_use]
    pub const fn dead_code_enabled(&self) -> bool { self.passes.contains(PassFlags::DEAD_CODE) }

    /// Check if escape analysis is enabled.
    #[must_use]
    pub const fn escape_analysis_enabled(&self) -> bool {
        self.passes.contains(PassFlags::ESCAPE_ANALYSIS)
    }

    /// Check if function inlining is enabled.
    #[must_use]
    pub const fn inlining_enabled(&self) -> bool { self.passes.contains(PassFlags::INLINING) }

    /// Check if reference count optimization is enabled.
    #[must_use]
    pub const fn refcount_opt_enabled(&self) -> bool {
        self.passes.contains(PassFlags::REFCOUNT_OPT)
    }

    /// Check if single-use function inlining is enabled.
    #[must_use]
    pub const fn inline_single_use_enabled(&self) -> bool {
        self.passes.contains(PassFlags::INLINE_SINGLE_USE)
    }

    /// No optimization (level 0).
    #[must_use]
    pub const fn level0() -> Self {
        Self {
            level: 0,
            max_iterations: 1,
            max_inline_size: 20,
            inline_threshold: 10,
            passes: PassFlags::empty(),
        }
    }

    /// Basic optimization (level 1): constant folding and DCE.
    #[must_use]
    pub const fn level1() -> Self {
        Self {
            level: 1,
            max_iterations: 5,
            max_inline_size: 20,
            inline_threshold: 10,
            passes: PassFlags::CONSTANT_FOLDING.union(PassFlags::DEAD_CODE),
        }
    }

    /// Standard optimization (level 2): all optimizations (default).
    #[must_use]
    pub const fn level2() -> Self {
        Self {
            level: 2,
            max_iterations: 10,
            max_inline_size: 20,
            inline_threshold: 10,
            passes: PassFlags::CONSTANT_FOLDING
                .union(PassFlags::DEAD_CODE)
                .union(PassFlags::INLINING)
                .union(PassFlags::ESCAPE_ANALYSIS)
                .union(PassFlags::REFCOUNT_OPT)
                .union(PassFlags::INLINE_SINGLE_USE),
        }
    }

    /// Aggressive optimization (level 3): higher iteration count.
    #[must_use]
    pub const fn level3() -> Self {
        Self {
            level: 3,
            max_iterations: 20,
            max_inline_size: 30,
            inline_threshold: 15,
            passes: PassFlags::CONSTANT_FOLDING
                .union(PassFlags::DEAD_CODE)
                .union(PassFlags::INLINING)
                .union(PassFlags::ESCAPE_ANALYSIS)
                .union(PassFlags::REFCOUNT_OPT)
                .union(PassFlags::INLINE_SINGLE_USE),
        }
    }
}

impl Default for OptimizerConfig {
    fn default() -> Self { Self::level2() }
}
