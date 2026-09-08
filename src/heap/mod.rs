//! Priority queues used by the neighbor-joining engines.
//!
//! [`MinHeap`] is a plain binary heap whose sift rules match the reference
//! implementation exactly, so that entries with equal keys are popped in the
//! same order. [`ArrayHeap`] is the external-memory heap used by the
//! disk-backed engine.

mod array_heap;
mod binary_heap;

pub use array_heap::{ArrayHeap, ArrayHeapConfig, Pair};
pub use binary_heap::MinHeap;
