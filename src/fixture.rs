use chrono::{DateTime, Utc};
use ethportal_api::{
    types::content_value::ContentValue, utils::bytes::hex_encode, BeaconContentKey,
    BeaconContentValue, OverlayContentKey,
};

#[derive(Debug)]
pub struct FixtureEntry {
    data_type: &'static str,
    content_key: BeaconContentKey,
    content_value: BeaconContentValue,
    updated_at: DateTime<Utc>,
}

impl FixtureEntry {
    pub fn new(
        data_type: &'static str,
        content_key: BeaconContentKey,
        content_value: BeaconContentValue,
    ) -> Self {
        Self {
            data_type,
            content_key,
            content_value,
            updated_at: Utc::now(),
        }
    }

    pub fn to_yaml_string(&self) -> String {
        format!(
            "# {}\n# Last updated: {}\n- content_key: \"{}\"\n  content_value: \"{}\"\n",
            self.data_type,
            self.updated_at.format("%Y-%m-%d"),
            self.content_key.to_hex(),
            hex_encode(self.content_value.encode()),
        )
    }
}
