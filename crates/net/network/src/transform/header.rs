//! Transformation hooks applied to block headers.

use reth_primitives_traits::BlockHeader;

/// Transforms a batch of headers downloaded from a peer before validation and persistence.
#[async_trait::async_trait]
pub trait HeaderTransform<H: BlockHeader>: std::fmt::Debug + Send + Sync {
    /// Applies the transformation to downloaded headers.
    async fn map(&self, headers: Vec<H>) -> Vec<H>;
}

#[async_trait::async_trait]
impl<H: BlockHeader> HeaderTransform<H> for () {
    async fn map(&self, headers: Vec<H>) -> Vec<H> {
        headers
    }
}

/// Transforms a stored header before it is returned to a peer.
#[async_trait::async_trait]
pub trait HeaderResponseTransform<H: BlockHeader>: std::fmt::Debug + Send + Sync {
    /// Applies the transformation to an outgoing header.
    async fn map(&self, header: H) -> H;
}

#[async_trait::async_trait]
impl<H: BlockHeader> HeaderResponseTransform<H> for () {
    async fn map(&self, header: H) -> H {
        header
    }
}
