//! Operations for cache maintenance.
//!
//! Since the agent that performs data access is separate from the agent that performs instruction
//! fetches, operations that change instructions must sync one's cache to the other. There are three
//! relevant caches that must be in sync for updates to instructions to become visible:
//!
//! - The data cache (d-cache), in which the changes are initially made.
//! - The instruction cache (i-cache), used by the instruction fetch pipeline.
//! - The branch prediction subsystem (the term "cache" is used loosely here, but this also has to
//!   be synced).
//!
//! Therefore, for an instruction update to take effect in a uniprocessor setting, the following
//! operations must be made:
//!
//! 1. The instruction must be written to the data cache. (e.g. via [`std::ptr::write_volatile`])
//! 2. The changes in the data cache must synced far enough that the i-cache sees it when it queries
//!    main memory. (via [`cache::clean_dcache_to_unification`])
//! 3. The instruction cache must read any changes from main memory. (via
//!    [`cache::invalidate_icache`])
//! 4. Any branch predictions for the instruction must be cleared, since they're now invalid. (this
//!    is handled by the previous function).

use std::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheTarget {
	/// Performs an operation on the cache line containing the given address.
	///
	/// Since cache lines are 32 bytes wide, the CPU will ignore the lower 5 bits of the
	/// address.
	Address(usize),
}

/// Ensure the visibility of an instruction update for a uniprocessor.
pub fn sync_instruction(target: CacheTarget) {
	clean_dcache_to_unification(target);
	invalidate_icache(target);
}

/// Syncs the given portion of data cache with main memory such that any changes made to this
/// cache are visible to other caches when they access main memory.
///
/// The cache is cleaned to the Point of Unification: other subsystems of the processor (such
/// as other caches and translation table walks) are guaranteed to see the changes, but the
/// changes aren't guaranteed to be visible to external agents that can access the memory.
#[inline]
fn clean_dcache_to_unification(target: CacheTarget) {
	unsafe {
		match target {
			CacheTarget::Address(addr) => {
				// Perform a Data Cache Clean by MVA to PoU.
				asm!(
					"mcr p15, 0, {mva}, c7, c11, 1",
					"dsb",
					mva = in(reg) addr,
					options(nostack, preserves_flags),
				);
			}
		}
	}
}

/// Invalidates the CPU instruction cache, so that any changes from main memory are synced into
/// the i-cache.
///
/// Branch predictors are also invalidated.
#[inline]
fn invalidate_icache(target: CacheTarget) {
	unsafe {
		match target {
			CacheTarget::Address(base) => {
				// Perform an Instruction Cache Invalidate by MVA to PoU.
				// Then perform a Branch Predictor Invalidate by MVA.
				asm!(
					"mcr p15, 0, {mva}, c7, c5, 1", // ICIMVAU
					"mcr p15, 0, {mva}, c7, c5, 7", // BPIMVA
					"dsb", // ensure invalidation is completed
					"isb", // ensure instruction fetch sees the change
					mva = in(reg) base,
					options(nostack, preserves_flags),
				);
			}
		}
	}
}
