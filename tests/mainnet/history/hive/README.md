# Hive History Test Formats

The current expectation is all test files are
- yaml
- include headers by hash, headers by number, bodies, and receipts in this respective order
- folder names should describe the test
- test file names should follow the format `<block_number>.yaml`

Test Categories
- success: these are expected to pass
- invalid proofs
- tbd


## Header Validation
To validate post-capella headers, we need access to HistoricalSummaries.
Hive will seed the HistoricalSummaries for the History tests using `portal_beaconStore`.
Historical Summaries should follow the `fork_digest_historical_summaries_<epoch>.ssz_snappy` naming scheme.
Unless otherwise specified, the tests will use the historical summaries with the highest epoch in this folder.

If tests require a specific historical summaries, it should be specified in this README. Else we should only store the latest historical summaries needed for testing in this folder.
