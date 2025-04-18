# Proofs

### Test assets for post-merge proof (HeaderWithProof) generation
- Each subdirectory is named by it's block height and contains assets needed to test proof generation utils
- pre-capella blocks:
  - `block.ssz` -> ssz encoded beacon block
  - `historical_batch.ssz` -> ssz encoded historical batch
- pre-deneb blocks (17034870, 17042287, 17062257) and pre-pectra (22162263):
  - `block.ssz` -> ssz encoded beacon block
  - `beacon_state.ssz` -> ssz encoded beacon state
