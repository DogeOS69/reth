//! Contains RPC handler implementations specific to endpoints that call/execute within evm.

use crate::EthApi;
use reth_rpc_convert::RpcConvert;
use reth_rpc_eth_api::{
    helpers::{estimate::EstimateCall, Call, EthCall},
    FromEvmError, RpcNodeCore,
};
use reth_rpc_eth_types::EthApiError;

impl<N, Rpc> EthCall for EthApi<N, Rpc>
where
    N: RpcNodeCore,
    EthApiError: FromEvmError<N::Evm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = EthApiError, Evm = N::Evm>,
{
}

impl<N, Rpc> Call for EthApi<N, Rpc>
where
    N: RpcNodeCore,
    EthApiError: FromEvmError<N::Evm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = EthApiError, Evm = N::Evm>,
{
    #[inline]
    fn call_gas_limit(&self) -> u64 {
        self.inner.gas_cap()
    }

    #[inline]
    fn max_simulate_blocks(&self) -> u64 {
        self.inner.max_simulate_blocks()
    }

    #[inline]
    fn evm_memory_limit(&self) -> u64 {
        self.inner.evm_memory_limit()
    }
}

impl<N, Rpc> EstimateCall for EthApi<N, Rpc>
where
    N: RpcNodeCore,
    EthApiError: FromEvmError<N::Evm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = EthApiError, Evm = N::Evm>,
{
}

#[cfg(test)]
mod tests {
    use crate::EthApi;
    use alloy_rpc_types_eth::{
        simulate::{SimBlock, SimulatePayload},
        Bundle, TransactionRequest,
    };
    use reth_evm_ethereum::EthEvmConfig;
    use reth_network_api::noop::NoopNetwork;
    use reth_provider::test_utils::NoopProvider;
    use reth_rpc_eth_api::helpers::{EthCall, SpawnBlocking};
    use reth_transaction_pool::test_utils::testing_pool;
    use std::time::Duration;

    /// `eth_simulateV1` and `eth_callMany` execute on the blocking pool, so they must wait for a
    /// blocking IO permit like `eth_call` instead of running unbounded.
    #[tokio::test]
    async fn simulate_and_call_many_wait_for_blocking_io_permit() {
        let eth_api = EthApi::builder(
            NoopProvider::default(),
            testing_pool(),
            NoopNetwork::default(),
            EthEvmConfig::mainnet(),
        )
        .build();

        // take every permit so the requests have to wait for one
        let guard = eth_api.blocking_io_task_guard().clone();
        let permits =
            guard.clone().acquire_many_owned(guard.available_permits() as u32).await.unwrap();

        let simulate = eth_api.simulate_v1(
            SimulatePayload { block_state_calls: vec![SimBlock::default()], ..Default::default() },
            None,
        );
        let call_many = eth_api.call_many(
            vec![Bundle {
                transactions: vec![TransactionRequest::default()],
                block_override: None,
            }],
            None,
            None,
        );
        tokio::pin!(simulate, call_many);

        // The noop provider has no blocks, so without a permit check both requests would fail
        // right away instead of waiting.
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut simulate).await.is_err(),
            "eth_simulateV1 must wait for a blocking IO permit"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut call_many).await.is_err(),
            "eth_callMany must wait for a blocking IO permit"
        );

        drop(permits);
        assert!(tokio::time::timeout(Duration::from_secs(5), simulate).await.is_ok());
        assert!(tokio::time::timeout(Duration::from_secs(5), call_many).await.is_ok());
    }
}
