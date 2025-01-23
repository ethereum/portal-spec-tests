mod client;
mod fixture;

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use client::BeaconClient;
use tracing::info;

// This script fetches various Ethereum beacon chain light client data and historical summaries
// from a Consensus Layer endpoint and saves them as YAML test fixtures.
//
// # Usage
// ```bash
// # Set required environment variables
// export PANDAOPS_CLIENT_ID="your_client_id"
// export PANDAOPS_CLIENT_SECRET="your_client_secret"
//
// cargo run
// ```
//
// This script is run once a month automatically as a github workflow, and
// creates a pr with the updated test fixtures.
// You can trigger this workflow to run manually by going to the actions tab in Github.
//
// This code is largely based off the portal-bridge codebase.
// todo: add bootstraps older than 4 months for tests

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting fixture update process");
    dotenv::dotenv().ok();

    let client_id = env::var("PANDAOPS_CLIENT_ID").context("PANDAOPS_CLIENT_ID not found")?;
    let client_secret =
        env::var("PANDAOPS_CLIENT_SECRET").context("PANDAOPS_CLIENT_SECRET not found")?;

    let client = BeaconClient::new(&client_id, &client_secret)?;
    // Update latest test fixtures
    let latest_data = fetch_latest_data(&client).await?;
    update_fixture(&latest_data, "test_data.yaml")?;

    info!("Successfully updated fixture");
    Ok(())
}

async fn fetch_latest_data(client: &BeaconClient) -> Result<String> {
    let bootstrap = client.get_light_client_bootstrap().await?;
    let finality = client.get_light_client_finality_update().await?;
    let optimistic = client.get_light_client_optimistic_update().await?;
    let updates = client.get_light_client_updates_by_range().await?;
    let historical = client.get_historical_summaries_with_proof().await?;

    let yaml_str = format!(
        "# Deneb test data for Portal Hive Beacon tests\n# Last updated: {}\n\n{}\n{}\n{}\n{}\n{}\n",
        chrono::Utc::now().format("%Y-%m-%d"),
        bootstrap.to_yaml_string(),
        finality.to_yaml_string(),
        optimistic.to_yaml_string(),
        updates.to_yaml_string(),
        historical.to_yaml_string(),
    );

    Ok(yaml_str)
}

fn update_fixture(data: &str, file_name: &str) -> Result<()> {
    let fixture_path = PathBuf::from("tests/mainnet/beacon_chain/hive").join(file_name);

    if !fixture_path.exists() {
        return Err(anyhow::anyhow!(
            "Fixture path not found: {:?}",
            fixture_path
        ));
    }

    fs::write(&fixture_path, data)?;
    Ok(())
}
