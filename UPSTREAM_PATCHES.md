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

## Trie unwind revert ordering

- **Scope:** `reth-provider` `DatabaseProvider::unwind_trie_state_from`.
- **Source:** focused backport of the `unwind_trie_state_from` reorder from upstream Reth commit
  `05ba48120a0ce8d6beb65f0b2a7ad4710f4f2915` (PR #27615).
- **Purpose:** compute the trie revert for the unwound range before account and storage hashing are
  unwound. On a changeset cache miss the revert is derived from the hashed state, and computing it
  from already-reverted hashed state left branch nodes created in the unwound blocks in the trie
  tables, giving a wrong state root on the next block.
- **Compatibility:** only the reorder is ported, using this branch's `ChangesetCache` API. The rest
  of PR #27615 targets the post-2.0 storage-overlay changeset cache and persistence frontiers, which
  this branch does not have, and its test depends on them. The branch adds
  `unwind_trie_state_with_cold_changeset_cache` and `unwind_tip_trie_state_with_cold_changeset_cache`
  (a two-block and a tip-only unwind), which fail on the old order.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #27615.

## Stale persisted forkchoice head

- **Scope:** `reth-engine-tree` `EngineApiTreeHandler::find_canonical_header`.
- **Source:** focused backport of upstream Reth commit `444e867155c2a7a4c028e479ca7201b0b6871579`
  (PR #27429).
- **Purpose:** a header found on disk by hash is only treated as canonical if it is not above the
  in-memory canonical head and matches the canonical hash at its height. Reorged-out blocks stay on
  disk until persistence removes them, so an FCU to such a hash could otherwise be treated as
  already canonical and skip making it the head again.
- **Compatibility:** `find_canonical_header` and the two engine tree unit tests match upstream.
  Dropped: the `is_consistent_forkchoice_state` hunk (that function comes from PR #27247, not on
  this branch) and the end-to-end tests in `crates/ethereum/node/tests/e2e/forkchoice.rs` (file and
  e2e setup hooks not on this branch). The branch adds
  `test_fcu_rejects_reorged_out_safe_and_finalized_with_pending_disk_reorg`: the same check now
  makes a reorged-out safe or finalized hash an invalid forkchoice state instead of being accepted.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #27429.

## RPC blocking IO bounds

- **Scope:** `reth-rpc-eth-api` blocking task helpers, `eth_call`, `eth_simulateV1`,
  `eth_callMany` and `eth_getProof`, and the `reth-rpc` `eth_getLogs` range scan.
- **Source:** backport of upstream Reth commits `de401d9b403481fdfe86a6d1841f77d460f41fc9`
  (PR #26776), `f2d2bd23306801f7ba8f7380ab9740dead844c4a` (PR #24503),
  `b7a7a8a7295c4ea841d5960a7e39a473e7d7cebe` (PR #24499),
  `1f94488d1e42866d3f8280702a546d42a7b288af` (PR #26999),
  `e63ec720ac2d11d9ff8694e54b22ac88b9714381` (PR #27106),
  `e88a669b1be89ef335e457d5093f8be61fc293ac` (PR #27107) and
  `055fafc723d1df096e551a14a80e76d2483e97fa` (PR #27000).
- **Purpose:** bound public RPC work on the blocking pool. Blocking IO tasks whose caller has gone
  skip or abandon their work; `eth_simulateV1`, `eth_callMany` and `eth_getLogs` range scans take a
  blocking IO permit like `eth_call`; and the `eth_call`/simulate/callMany blocking IO permits and
  the `eth_getProof` tracing permit are held inside the spawned task, so a dropped request no
  longer frees its permit while the work keeps running. `eth_getLogs` range scans run window by
  window, and each parallel receipt worker takes its own blocking IO permit (or runs on the scan's
  task when none is free), so a cancelled scan no longer leaves workers running without one.
- **Compatibility:** PRs #26776, #24503, #24499 and #26999 match upstream exactly. From PR #27106
  the `bal.rs` hunk is dropped: upstream moves an existing permit, but this branch's
  `get_block_access_list` takes none. Its `eth_simulateV1` hunk is resolved by hand onto this
  branch's older simulate setup with the same effect. From PR #27107 the `get_multi_proof` hunk is
  dropped; that method does not exist on this branch. From PR #27000 the per-block loop body and
  the batch capacity keep this branch's code, which predates upstream's converter argument,
  retry range (#23818) and capacity bound (#26971); its test expects the log-limit error at the
  block that exceeded the limit, which is what this branch reports. Upstream adds tests for #26999
  and #27000; the branch adds `simulate_and_call_many_wait_for_blocking_io_permit`,
  `test_cancelled_range_scan_workers_hold_blocking_io_permits` and
  `test_spawn_blocking_io_fut_drops_cancelled_future` in `reth-rpc`.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PRs
  #26776, #24503, #24499, #26999, #27106, #27107 and #27000.

## Log subscription broadcast lag

- **Scope:** `reth-rpc` `eth_subscribe("logs")` stream.
- **Source:** exact backport of upstream Reth commit `6e4009eed404816b3d33c2869e2022189af17df7`
  (PR #23561).
- **Purpose:** build the log stream on `canonical_state_stream()`, which skips broadcast lag, instead
  of a raw `BroadcastStream` whose `Err(Lagged)` item hit an `expect` and panicked the subscription
  task. A slow WS client or a burst of canonical updates could silently end a client's log feed.
- **Compatibility:** the diff matches upstream line for line. Upstream has no test for this change;
  the branch adds `log_stream_survives_lagged_canonical_state` in `pubsub.rs`, which fails on the
  old code.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #23561. Drop the added test with it, or keep it if upstream still lacks one.

## Syncing subscription close

- **Scope:** `reth-rpc` `eth_subscribe("syncing")` task.
- **Source:** exact backport of upstream Reth commit `d85d9deaef3b63161a339e2049f1917574eb1d2a`
  (PR #27455).
- **Purpose:** end the syncing subscription task when its sink closes (unsubscribe or disconnect).
  The task only sends when the sync status changes, so on a synced node it previously never noticed
  a closed sink and lived forever, holding a canonical-state receiver.
- **Compatibility:** none beyond upstream; the diff matches upstream line for line.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #27455.

## Pending inbound throttle at capacity

- **Scope:** `reth-network` `PeersManager` inbound admission.
- **Source:** exact backport of upstream Reth commit `bc98f383d23c68ccd3b9f067ba199e825eae737f`
  (PR #27842).
- **Purpose:** when the inbound slots are full and trusted peer ids are configured, pending inbound
  connections admitted for possible trusted peers are now IP-throttled too, so one host can no
  longer hold every pending slot. Throttle entries move to a bounded LRU separate from security
  bans and only apply to global IPs.
- **Compatibility:** none beyond upstream; the diff matches upstream line for line.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #27842.

## P2P ping validation and pacing

- **Scope:** `reth-eth-wire` `P2PStream` Ping/Pong handling, and the matching `reth-network` error
  classification.
- **Source:** backport of upstream Reth commits `20fab813c08ab9f5c7f7a2db1d3c304d717fe46c`
  (PR #26698) and `ae81966b68e71028b9efeead3e4a9bcb38ffe049` (PR #26702).
- **Purpose:** reject Ping/Pong frames whose payload is not the 1-byte empty list before
  decompressing them, and limit incoming pings to a burst of 5 refilled at one per second. A peer
  that floods pings without reading can no longer grow the unbounded outgoing Pong queue.
- **Compatibility:** the production code matches upstream exactly; `P2PStream::new` lacks the
  upstream `compress_scratch` field, which comes from a later perf commit. The upstream tests rely
  on test helpers from later perf commits (`FlushCountingTransport`, `compress_frame`,
  `SNAPPY_PING_MESSAGE`/`SNAPPY_PONG_MESSAGE`). They run on a minimal `IncomingFramesTransport`,
  build compressed frames with the Snappy encoder directly instead of `compress_frame`, and use the
  byte-identical `P2PMessage::{Ping, Pong}` encodings instead of the constants. The two upstream
  flush-batching tests that cover those perf commits are omitted. The branch adds
  `test_invalid_ping_pong_payload_is_protocol_breach` in `reth-network`, which upstream's tests
  check only for `TooManyPings`.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PRs
  #26698 and #26702.

## RLPx Hello identity check

- **Scope:** `reth-network` session handshake (`authenticate_stream`).
- **Source:** backport of upstream Reth commit `1c6945c561d37a79857c97ee65c77338223ef639`
  (PR #26639).
- **Purpose:** bind every session to the node id the ECIES handshake authenticated. A remote whose
  `Hello` announces a different node id is disconnected with `UnexpectedHandshakeIdentity`, so it
  cannot take over a trusted peer's id, its peer record, or its extra-protocol (scroll-wire)
  connection.
- **Compatibility:** the production code matches upstream exactly. The upstream session tests pass a
  max message size to the pending-session functions (PR #22668), which this branch does not carry;
  that argument is dropped from the tests.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #26639.

## Trusted peer revocation

- **Scope:** `reth-network` trusted peer set and trusted peer resolver.
- **Source:** exact backport of upstream Reth commit
  `b05e68db85fb29c4eaa707728171acb0fd0cd23d` (PR #24406), and a focused backport of upstream Reth
  commit `f9858796a53be55d5861b13e0eb6b22d8af85f9e` (PR #27948).
- **Purpose:** removing a trusted peer always drops its id from the trusted set and the resolver,
  even when the peer has no record or a non-trusted record, so a revoked peer no longer passes the
  `--trusted-only` admission check.
- **Compatibility:** the function body matches upstream. The upstream test's admin ban assertions
  are omitted because this branch does not carry PR #25140 (`admin` ban and unban methods).
- **Removal condition:** remove the backport when the selected upstream Reth base contains PRs
  #24406 and #27948.
