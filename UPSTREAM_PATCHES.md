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
  `unwind_trie_state_with_cold_changeset_cache`, which fails on the old order.
- **Removal condition:** remove the backport when the selected upstream Reth base contains PR
  #27615.
