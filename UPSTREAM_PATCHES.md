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

## Bounded session queues

- **Scope:** `reth-eth-wire` RLPx multiplexer (`multiplex.rs`), `P2PStream` and
  `SharedCapabilities::find_by_offset`, the `ProtocolIngressLimits` type, and the `reth-network`
  `ConnectionHandler::inbound_limits` hook and error classification.
- **Source:** backport of upstream Reth commits `01cbe9bdada95a3267a6b75bd54b27b6452ab97a`
  (PR #25031), `dd637cefe1f79f6e4815a2389a36412296cde4ad` (PR #26660) and its prerequisite
  `3a83ccc546bca3a90eb492fc0d115e6065aecb1a` (PR #26654).
- **Purpose:** bound the queues between the RLPx connection and its protocols. The outbound mux
  buffer gets a soft byte cap and polls protocols in turn (#25031). Each inbound protocol queue,
  primary (eth) and satellite (scroll-wire) alike, is bounded by message count and bytes, with a
  shared per-connection byte budget, both during the eth `Status` handshake and after it; the
  handshake loop polls the handshake and satellites before reading more frames, and yields every
  32 frames (#26660). A full queue ends the session with `SubprotocolInboundBufferFull`. Message IDs
  past the last negotiated capability's range are rejected instead of being routed to that
  capability (#26654), so `P2PStream`'s unknown-ID check, the per-protocol frame limit and the
  multiplexer's routing all agree on which protocol owns an ID.
- **Compatibility:** #25031 and #26654 match upstream exactly. #26654 is committed after #26660 on
  this branch (it was added after the branch was pushed); upstream has it first. #26660 matches
  upstream except that the byte budget uses `AtomicUsize::fetch_update` in place of `try_update`
  (Rust 1.95; this branch and its downstream builds use 1.93), and its two `P2PStream` tests use a
  local frame helper and the existing `IncomingFramesTransport` because `compress_frame` comes from
  a later perf commit. `ConnectionHandler::inbound_limits` has a default, so the scroll-wire handler
  keeps compiling and gets the default limits (1,024 messages, 32 MiB). The branch adds
  `pre_status_scroll_wire_queue_is_bounded_by_default_limits`, which upstream's tests do not cover
  with the real eth handshake. The branch is stacked on the RLPx Hello identity check and P2P ping
  validation and pacing backports.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PRs
  #25031, #26654 and #26660.
