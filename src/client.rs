use anyhow::{ensure, Result};
use ethportal_api::{
    consensus::{
        beacon_state::BeaconStateDeneb,
        historical_summaries::{HistoricalSummariesStateProof, HistoricalSummariesWithProof},
    },
    light_client::{
        bootstrap::LightClientBootstrapDeneb, finality_update::LightClientFinalityUpdateDeneb,
        optimistic_update::LightClientOptimisticUpdateDeneb, update::LightClientUpdateDeneb,
    },
    types::{
        consensus::fork::ForkName,
        content_key::beacon::{
            HistoricalSummariesWithProofKey, LightClientBootstrapKey, LightClientFinalityUpdateKey,
            LightClientOptimisticUpdateKey, LightClientUpdatesByRangeKey,
        },
        content_value::beacon::{
            ForkVersionedHistoricalSummariesWithProof, ForkVersionedLightClientUpdate,
            LightClientUpdatesByRange,
        },
    },
    utils::bytes::hex_decode,
    BeaconContentKey, BeaconContentValue,
};
use reqwest::Client;
use serde_yaml::Value;
use ssz_types::VariableList;
use tracing::info;

use crate::fixture::FixtureEntry;

// Pandaops consensus endpoint.
const BASE_CL_ENDPOINT: &str = "https://nimbus-geth.mainnet.eu1.ethpandaops.io";
// The number of slots in an epoch.
const SLOTS_PER_EPOCH: u64 = 32;
/// The number of slots in a sync committee period.
const SLOTS_PER_PERIOD: u64 = SLOTS_PER_EPOCH * 256;
// Beacon chain mainnet genesis time: Tue Dec 01 2020 12:00:23 GMT+0000
pub const BEACON_GENESIS_TIME: u64 = 1606824023;
/// The historical summaries proof always has a length of 5 hashes.
const HISTORICAL_SUMMARIES_PROOF_LENGTH: usize = 5;

pub struct BeaconClient {
    client: Client,
}

impl BeaconClient {
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self> {
        info!("Creating new BeaconClient");
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "CF-Access-Client-ID",
            reqwest::header::HeaderValue::from_str(client_id)?,
        );
        headers.insert(
            "CF-Access-Client-Secret",
            reqwest::header::HeaderValue::from_str(client_secret)?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| anyhow::anyhow!("Failed to build HTTP client"))?;

        Ok(Self { client })
    }

    async fn get_finalized_root(&self) -> Result<String> {
        info!("Fetching finalized root");
        let url = format!("{}/eth/v1/beacon/blocks/finalized/root", BASE_CL_ENDPOINT);
        let response = self.client.get(url).send().await?;
        let json_data = response.error_for_status()?.json::<Value>().await?;
        Ok(json_data["data"]["root"].as_str().unwrap().to_string())
    }

    pub async fn get_light_client_bootstrap(&self) -> Result<FixtureEntry> {
        info!("Fetching light client bootstrap data");
        let block_root = self.get_finalized_root().await?;
        let url = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/{}",
            BASE_CL_ENDPOINT, block_root
        );
        let response = self.client.get(url).send().await?;
        let json_data = response
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        let result: serde_json::Value = json_data["data"].clone();
        let content_key = BeaconContentKey::LightClientBootstrap(LightClientBootstrapKey {
            block_hash: <[u8; 32]>::try_from(hex_decode(&block_root)?).unwrap(),
        });
        let bootstrap: LightClientBootstrapDeneb = serde_json::from_value(result.clone())?;
        let content_value = BeaconContentValue::LightClientBootstrap(bootstrap.into());

        Ok(FixtureEntry::new(
            "Light Client Bootstrap",
            content_key,
            content_value,
        ))
    }

    pub async fn get_light_client_finality_update(&self) -> Result<FixtureEntry> {
        info!("Fetching light client finality update");
        let url = format!(
            "{}/eth/v1/beacon/light_client/finality_update",
            BASE_CL_ENDPOINT
        );
        let response = self.client.get(url).send().await?;
        let json_data = response
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        let result: serde_json::Value = json_data["data"].clone();
        let update: LightClientFinalityUpdateDeneb = serde_json::from_value(result.clone())?;
        let new_finalized_slot = update.finalized_header.beacon.slot;
        let content_key = BeaconContentKey::LightClientFinalityUpdate(
            LightClientFinalityUpdateKey::new(new_finalized_slot),
        );
        let content_value = BeaconContentValue::LightClientFinalityUpdate(update.into());

        Ok(FixtureEntry::new(
            "Light Client Finality Update",
            content_key,
            content_value,
        ))
    }

    pub async fn get_light_client_optimistic_update(&self) -> Result<FixtureEntry> {
        info!("Fetching light client optimistic update");
        let url = format!(
            "{}/eth/v1/beacon/light_client/optimistic_update",
            BASE_CL_ENDPOINT
        );
        let response = self.client.get(url).send().await?;
        let json_data = response
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        let result: serde_json::Value = json_data["data"].clone();
        let update: LightClientOptimisticUpdateDeneb = serde_json::from_value(result.clone())?;
        let content_key = BeaconContentKey::LightClientOptimisticUpdate(
            LightClientOptimisticUpdateKey::new(update.signature_slot),
        );
        let content_value = BeaconContentValue::LightClientOptimisticUpdate(update.into());

        Ok(FixtureEntry::new(
            "Light Client Optimistic Update",
            content_key,
            content_value,
        ))
    }

    pub async fn get_light_client_updates_by_range(&self) -> Result<FixtureEntry> {
        info!("Fetching light client updates by range");
        let start_period = get_start_period().await?;
        let count = 1;

        let url = format!(
            "{}/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            BASE_CL_ENDPOINT, start_period, count
        );
        let response = self.client.get(url).send().await?;
        let json_data = response
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        let update: LightClientUpdateDeneb = serde_json::from_value(json_data[0]["data"].clone())?;
        let fork_versioned_update = ForkVersionedLightClientUpdate {
            fork_name: ForkName::Deneb,
            update: update.into(),
        };
        let content_value = BeaconContentValue::LightClientUpdatesByRange(
            LightClientUpdatesByRange(VariableList::from(vec![fork_versioned_update])),
        );
        let content_key =
            BeaconContentKey::LightClientUpdatesByRange(LightClientUpdatesByRangeKey {
                start_period,
                count,
            });

        Ok(FixtureEntry::new(
            "Light Client Updates By Range",
            content_key,
            content_value,
        ))
    }

    pub async fn get_historical_summaries_with_proof(&self) -> Result<FixtureEntry> {
        info!("Fetching historical summaries with proof");
        let url = format!("{}/eth/v2/debug/beacon/states/finalized", BASE_CL_ENDPOINT);
        let response = self.client.get(url).send().await?;
        let json_data = response.error_for_status()?.text().await?;
        let beacon_state_val: serde_json::Value = serde_json::from_str(&json_data)?;
        let beacon_state: BeaconStateDeneb =
            serde_json::from_value(beacon_state_val["data"].clone())?;
        let state_epoch = beacon_state.slot / SLOTS_PER_EPOCH;
        let historical_summaries_proof = beacon_state.build_historical_summaries_proof();

        ensure!(
            historical_summaries_proof.len() == HISTORICAL_SUMMARIES_PROOF_LENGTH,
            "Historical summaries proof length is not 5",
        );

        let historical_summaries = beacon_state.historical_summaries;
        let historical_summaries_with_proof = ForkVersionedHistoricalSummariesWithProof {
            fork_name: ForkName::Deneb,
            historical_summaries_with_proof: HistoricalSummariesWithProof {
                epoch: state_epoch,
                historical_summaries,
                proof: HistoricalSummariesStateProof::from(historical_summaries_proof),
            },
        };
        let content_key =
            BeaconContentKey::HistoricalSummariesWithProof(HistoricalSummariesWithProofKey {
                epoch: state_epoch,
            });
        let content_value =
            BeaconContentValue::HistoricalSummariesWithProof(historical_summaries_with_proof);

        Ok(FixtureEntry::new(
            "Historical Summaries With Proof",
            content_key,
            content_value,
        ))
    }
}

async fn get_start_period() -> Result<u64> {
    let now = std::time::SystemTime::now();
    let expected_current_period =
        expected_current_slot(BEACON_GENESIS_TIME, now) / SLOTS_PER_PERIOD;
    Ok(expected_current_period)
}

fn expected_current_slot(genesis_time: u64, now: std::time::SystemTime) -> u64 {
    let now = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let since_genesis = now - std::time::Duration::from_secs(genesis_time);

    since_genesis.as_secs() / 12
}
