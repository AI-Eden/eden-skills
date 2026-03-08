//! `.eden-managed` manifest support for agent directories.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const MANAGED_MANIFEST_FILE: &str = ".eden-managed";
const MANAGED_MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManagedSource {
    External,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedSkillRecord {
    pub source: ManagedSource,
    pub origin: String,
    pub installed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedManifest {
    pub version: u32,
    #[serde(default)]
    pub skills: BTreeMap<String, ManagedSkillRecord>,
}

impl Default for ManagedManifest {
    fn default() -> Self {
        Self {
            version: MANAGED_MANIFEST_VERSION,
            skills: BTreeMap::new(),
        }
    }
}

impl ManagedManifest {
    pub fn parse(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn record_install(
        &mut self,
        skill_id: &str,
        source: ManagedSource,
        origin: impl Into<String>,
    ) {
        self.skills.insert(
            skill_id.to_string(),
            ManagedSkillRecord {
                source,
                origin: origin.into(),
                installed_at: current_timestamp_rfc3339(),
            },
        );
    }

    pub fn remove_skill(&mut self, skill_id: &str) {
        self.skills.remove(skill_id);
    }

    pub fn skill(&self, skill_id: &str) -> Option<&ManagedSkillRecord> {
        self.skills.get(skill_id)
    }
}

pub fn external_install_origin() -> String {
    format!("host:{}", resolve_host_identity())
}

pub fn local_install_origin(environment: &str) -> String {
    environment.strip_prefix("docker:").map_or_else(
        || "local".to_string(),
        |container| format!("container:{container}"),
    )
}

fn resolve_host_identity() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown-host".to_string())
}

fn current_timestamp_rfc3339() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_unix_timestamp(seconds)
}

fn format_unix_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
