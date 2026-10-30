# Upstream Patch Queue

This branch is based on Reth `eb4c15e5e36d8776d46629beae4c0a69af7ab04f` (`v2.0.0`).

## RocksDB synchronous writes

- **Scope:** `reth-provider` RocksDB write transactions and batches.
- **Source:** exact backport of upstream Reth commit
  `3a136fc8c38221e060cbc31ef5c5fa345cf0e17a` (PR #23603).
- **Purpose:** set `WriteOptions::sync(true)` for transactions, explicit batches, auto-committed
  batches, and final batch commits so a successful commit is durable across a host crash.
- **Compatibility:** this changes write durability only and does not alter the Reth 2 / REVM 36
  dependency family or DogeOS storage encoding.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #23603 or an equivalent synchronous-write implementation.

## Header transformation hooks

- **Scope:** `reth-network` downloaded-header handling only.
- **Purpose:** provide temporary inbound geth/l2geth-to-Reth historical-header compatibility during
  the controlled pre-Tsuki crossover. Network response heuristics inspect the raw peer response;
  afterward, the asynchronous hook transforms downloaded headers before normal Reth downloader
  validation and persistence.
- **Protocol ownership:** the hook contains no DogeOS or Scroll rules. The configured DogeOS adapter
  owns any required signature restoration and must preserve header count and ordering.
- **Default behavior:** no transform is configured, so downloaded responses follow the upstream
  synchronous delivery path.
- **Serving behavior:** Reth never serves transformed headers; geth nodes must continue syncing
  historical headers from geth peers during the compatibility window.
- **Removal condition:** remove this patch after all nodes have migrated to Reth and the controlled
  compatibility window has ended.

## EIP-3607 empty-code sender validation

- **Scope:** `reth-transaction-pool` sender bytecode validation.
- **Source:** focused backport of upstream Reth commit
  `62808bd060e4c7398e3fb6df93881950b1433b18` (PR #26644).
- **Purpose:** treat `KECCAK_EMPTY` as no deployed code so genesis accounts declared with explicit
  empty code remain valid EIP-3607 senders.
- **Compatibility:** nonempty code remains rejected before Prague; after Prague, only valid EIP-7702
  delegation code remains accepted.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #26644 or equivalent EIP-3607 empty-code handling.

## RPC blocking IO bounds

- **Scope:** `reth-rpc-eth-api` blocking task helpers, `eth_call`, `eth_simulateV1`,
  `eth_callMany` and `eth_getProof`, and the `reth-rpc` `eth_getLogs` range scan.
- **Source:** backport of upstream Reth commits `de401d9b403481fdfe86a6d1841f77d460f41fc9`
  (PR #26776), `f2d2bd23306801f7ba8f7380ab9740dead844c4a` (PR #24503),
  `b7a7a8a7295c4ea841d5960a7e39a473e7d7cebe` (PR #24499),
  `1f94488d1e42866d3f8280702a546d42a7b288af` (PR #26999),
  `e63ec720ac2d11d9ff8694e54b22ac88b9714381` (PR #27106) and
  `e88a669b1be89ef335e457d5093f8be61fc293ac` (PR #27107).
- **Purpose:** bound public RPC work on the blocking pool. Blocking IO tasks whose caller has gone
  skip or abandon their work; `eth_simulateV1`, `eth_callMany` and `eth_getLogs` range scans take a
  blocking IO permit like `eth_call`; and the `eth_call`/simulate/callMany blocking IO permits and
  the `eth_getProof` tracing permit are held inside the spawned task, so a dropped request no
  longer frees its permit while the work keeps running.
- **Compatibility:** PRs #26776, #24503, #24499 and #26999 match upstream exactly. From PR #27106
  the `bal.rs` hunk is dropped: upstream moves an existing permit, but this branch's
  `get_block_access_list` takes none. Its `eth_simulateV1` hunk is resolved by hand onto this
  branch's older simulate setup with the same effect. From PR #27107 the `get_multi_proof` hunk is
  dropped; that method does not exist on this branch. Upstream adds a test only for #26999; the
  branch adds `simulate_and_call_many_wait_for_blocking_io_permit` in `reth-rpc`.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PRs
  #26776, #24503, #24499, #26999, #27106 and #27107.
