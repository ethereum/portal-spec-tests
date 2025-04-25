# Proofs

### Test assets for post-merge proof (HeaderWithProof) generation and verification
- Each subdirectory is named by it's block height and contains assets needed to test proof generation utils:
  - pre-capella blocks:
    - `block.ssz` -> ssz encoded beacon block
    - `historical_batch.ssz` -> ssz encoded historical batch
  - capella and onwards blocks:
    - `block.ssz` -> ssz encoded beacon block
    - `block_roots.ssz` -> ssz encoded beacon state `block_roots` field

- The `beacon_data` directory also holds the beacon state field `historical_summaries` at a specific slot which is required to verify the block header proofs:
  - `historical_summaries_at_slot_x.ssz` -> ssz encoded `historical_summaries` at slot number x.
