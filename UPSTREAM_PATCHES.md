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

- **Scope:** `reth-network` and `reth-node-builder` only.
- **Purpose:** expose generic asynchronous hooks for transforming headers after download and before
  serving a header response.
- **Protocol ownership:** the hooks contain no DogeOS or Scroll rules. Callers own signature
  extraction, verification, persistence, and restoration.
- **Default behavior:** both hooks are no-ops unless explicitly configured.
- **Required by:** Feynman-and-later DogeOS historical sync, where the wire representation carries
  a signed-header field that is not part of the canonical stored header.
- **Removal condition:** remove this patch once equivalent downloader and response transformation
  hooks are available in the selected upstream Reth release.
