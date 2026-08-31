//! Transformation hooks applied to downloaded block headers.

use reth_primitives_traits::BlockHeader;

/// Transforms a batch of headers downloaded from a peer after network response heuristics and
/// before the batch is delivered for downloader validation and persistence.
///
/// Implementations must return exactly one header for each input header and preserve the input
/// ordering. A transform that panics, never completes, or violates this contract can stall local
/// synchronization or cause the transformed response to fail validation.
#[async_trait::async_trait]
pub trait HeaderTransform<H: BlockHeader>: std::fmt::Debug + Send + Sync {
    /// Applies the transformation to downloaded headers.
    async fn map(&self, headers: Vec<H>) -> Vec<H>;
}
