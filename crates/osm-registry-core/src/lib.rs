use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub const REGISTRY_SCHEMA_VERSION: u8 = 1;
pub const MAX_BATCH: usize = 50;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum RegionLevel {
    Country = 0,
    Subregion = 1,
}

impl TryFrom<u8> for RegionLevel {
    type Error = RegistryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Country),
            1 => Ok(Self::Subregion),
            other => Err(RegistryError::InvalidLevel(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RegistryEntry {
    pub region: String,
    pub parent: Option<String>,
    pub level: RegionLevel,
    pub cid: String,
    pub source_url: String,
    pub checksum: String,
    pub version: u64,
    pub hosted: bool,
    pub timestamp: u64,
}

impl RegistryEntry {
    pub fn validate(&self) -> Result<(), RegistryError> {
        if self.region.trim().is_empty() {
            return Err(RegistryError::InvalidRegion);
        }

        match self.level {
            RegionLevel::Country => {
                if self.parent.as_ref().is_some_and(|p| !p.trim().is_empty()) {
                    return Err(RegistryError::InvalidParent {
                        region: self.region.clone(),
                        reason: "country entries must not have a registry parent".to_string(),
                    });
                }
            }
            RegionLevel::Subregion => {
                if self.parent.as_ref().is_none_or(|p| p.trim().is_empty()) {
                    return Err(RegistryError::InvalidParent {
                        region: self.region.clone(),
                        reason: "subregion entries require a registry parent".to_string(),
                    });
                }
            }
        }

        if self.cid.trim().is_empty() {
            return Err(RegistryError::InvalidCid);
        }
        if !self.source_url.starts_with("https://") {
            return Err(RegistryError::InvalidSourceUrl);
        }
        if self.checksum.len() != 32 || !self.checksum.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(RegistryError::InvalidChecksum);
        }
        if self.version == 0 {
            return Err(RegistryError::InvalidVersion);
        }
        if self.timestamp == 0 {
            return Err(RegistryError::InvalidTimestamp);
        }

        Ok(())
    }

    /// True when two writes describe the same region snapshot/status.
    ///
    /// Registration timestamp is deliberately excluded: replaying the same
    /// snapshot later must be idempotent rather than manufacturing a new
    /// registry version solely because the client observed it again.
    fn same_snapshot(&self, other: &Self) -> bool {
        self.region == other.region
            && self.parent == other.parent
            && self.level == other.level
            && self.cid == other.cid
            && self.source_url == other.source_url
            && self.checksum == other.checksum
            && self.version == other.version
            && self.hosted == other.hosted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Registry {
    pub schema_version: u8,
    /// Exactly one current entry per Geofabrik region path.
    ///
    /// The vector is kept globally ordered by timestamp descending, then
    /// source version descending, then region ascending.
    pub entries: Vec<RegistryEntry>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            schema_version: REGISTRY_SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationOutcome {
    pub inserted: usize,
    pub updated: usize,
    pub skipped_idempotent: usize,
}

impl Registry {
    pub fn register_batch(
        &mut self,
        entries: Vec<RegistryEntry>,
    ) -> Result<RegistrationOutcome, RegistryError> {
        if entries.is_empty() {
            return Err(RegistryError::BatchEmpty);
        }
        if entries.len() > MAX_BATCH {
            return Err(RegistryError::BatchTooBig {
                actual: entries.len(),
                max: MAX_BATCH,
            });
        }

        let mut regions = HashSet::with_capacity(entries.len());
        for entry in &entries {
            entry.validate()?;
            if !regions.insert(entry.region.as_str()) {
                return Err(RegistryError::DuplicateRegionInBatch {
                    region: entry.region.clone(),
                });
            }
        }

        // Work against a clone so any invalid/stale write fails the entire
        // batch without partially mutating registry state.
        let mut candidate = self.clone();
        let mut inserted = 0;
        let mut updated = 0;
        let mut skipped_idempotent = 0;

        for entry in entries {
            if let Some(index) = candidate
                .entries
                .iter()
                .position(|existing| existing.region == entry.region)
            {
                let existing = &candidate.entries[index];

                if existing.same_snapshot(&entry) {
                    skipped_idempotent += 1;
                    continue;
                }

                if existing.parent != entry.parent || existing.level != entry.level {
                    return Err(RegistryError::RegionShapeChanged {
                        region: entry.region,
                    });
                }

                if entry.timestamp <= existing.timestamp {
                    return Err(RegistryError::StaleTimestamp {
                        region: entry.region,
                        current: existing.timestamp,
                        incoming: entry.timestamp,
                    });
                }

                if entry.version < existing.version {
                    return Err(RegistryError::VersionDowngrade {
                        region: entry.region,
                        current: existing.version,
                        incoming: entry.version,
                    });
                }

                candidate.entries[index] = entry;
                updated += 1;
            } else {
                candidate.entries.push(entry);
                inserted += 1;
            }
        }

        sort_registry_entries(&mut candidate.entries);
        *self = candidate;

        Ok(RegistrationOutcome {
            inserted,
            updated,
            skipped_idempotent,
        })
    }

    pub fn by_region(&self, region: &str) -> Option<&RegistryEntry> {
        self.entries.iter().find(|entry| entry.region == region)
    }

    pub fn by_parent(&self, parent: &str) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.parent.as_deref() == Some(parent))
            .collect()
    }

    pub fn by_cid(&self, cid: &str) -> Vec<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.cid == cid)
            .collect()
    }

    pub fn latest_by_region(&self, region: &str) -> Option<&RegistryEntry> {
        self.by_region(region)
    }
}

fn sort_registry_entries(entries: &mut [RegistryEntry]) {
    entries.sort_by(|a, b| {
        b.timestamp
            .cmp(&a.timestamp)
            .then_with(|| b.version.cmp(&a.version))
            .then_with(|| a.region.cmp(&b.region))
    });
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryError {
    #[error("batch is empty")]
    BatchEmpty,
    #[error("batch size {actual} exceeds max {max}")]
    BatchTooBig { actual: usize, max: usize },
    #[error("batch contains region more than once: {region}")]
    DuplicateRegionInBatch { region: String },
    #[error("region is empty")]
    InvalidRegion,
    #[error("invalid parent for region {region}: {reason}")]
    InvalidParent { region: String, reason: String },
    #[error("invalid region level {0}")]
    InvalidLevel(u8),
    #[error("CID is empty")]
    InvalidCid,
    #[error("source URL must be https")]
    InvalidSourceUrl,
    #[error("checksum must be a 32-character MD5 hex string")]
    InvalidChecksum,
    #[error("version must be non-zero")]
    InvalidVersion,
    #[error("timestamp must be non-zero")]
    InvalidTimestamp,
    #[error("parent/level for existing region changed: {region}")]
    RegionShapeChanged { region: String },
    #[error(
        "stale registry timestamp for {region}: incoming {incoming} is not newer than current {current}"
    )]
    StaleTimestamp {
        region: String,
        current: u64,
        incoming: u64,
    },
    #[error(
        "registry version downgrade for {region}: incoming {incoming} is older than current {current}"
    )]
    VersionDowngrade {
        region: String,
        current: u64,
        incoming: u64,
    },
}

impl RegistryError {
    pub const fn code(&self) -> u32 {
        match self {
            Self::BatchEmpty => 1,
            Self::BatchTooBig { .. } => 2,
            Self::DuplicateRegionInBatch { .. } => 3,
            Self::InvalidRegion => 4,
            Self::InvalidParent { .. } => 5,
            Self::InvalidLevel(_) => 6,
            Self::InvalidCid => 7,
            Self::InvalidSourceUrl => 8,
            Self::InvalidChecksum => 9,
            Self::InvalidVersion => 10,
            Self::InvalidTimestamp => 11,
            Self::RegionShapeChanged { .. } => 12,
            Self::StaleTimestamp { .. } => 13,
            Self::VersionDowngrade { .. } => 14,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        region: &str,
        parent: Option<&str>,
        level: RegionLevel,
        cid: &str,
        version: u64,
        timestamp: u64,
    ) -> RegistryEntry {
        RegistryEntry {
            region: region.to_string(),
            parent: parent.map(str::to_string),
            level,
            cid: cid.to_string(),
            source_url: format!("https://download.geofabrik.de/{region}-latest.osm.pbf"),
            checksum: "426bd510159627dc139d4d0ad3bc6acd".to_string(),
            version,
            hosted: true,
            timestamp,
        }
    }

    #[test]
    fn default_registry_uses_schema_v1() {
        let registry = Registry::default();
        assert_eq!(registry.schema_version, REGISTRY_SCHEMA_VERSION);
        assert!(registry.entries.is_empty());
    }

    #[test]
    fn batch_registration_is_idempotent_for_same_snapshot_even_with_later_observation() {
        let ethiopia = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "zDv-ethiopia",
            1_789_685_417,
            1_789_700_000,
        );
        let mut replay = ethiopia.clone();
        replay.timestamp += 100;

        let mut registry = Registry::default();

        let first = registry.register_batch(vec![ethiopia.clone()]).unwrap();
        assert_eq!(
            first,
            RegistrationOutcome {
                inserted: 1,
                updated: 0,
                skipped_idempotent: 0
            }
        );

        let second = registry.register_batch(vec![replay]).unwrap();
        assert_eq!(
            second,
            RegistrationOutcome {
                inserted: 0,
                updated: 0,
                skipped_idempotent: 1
            }
        );
        assert_eq!(registry.entries, vec![ethiopia]);
    }

    #[test]
    fn newer_snapshot_replaces_region_instead_of_creating_history_row() {
        let old = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-old",
            100,
            1_000,
        );
        let new = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-new",
            200,
            2_000,
        );

        let mut registry = Registry::default();
        registry.register_batch(vec![old]).unwrap();
        let outcome = registry.register_batch(vec![new.clone()]).unwrap();

        assert_eq!(outcome.inserted, 0);
        assert_eq!(outcome.updated, 1);
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.by_region("ethiopia"), Some(&new));
        assert!(registry.by_cid("cid-old").is_empty());
        assert_eq!(registry.by_cid("cid-new"), vec![&new]);
    }

    #[test]
    fn stale_write_fails_without_partial_batch_mutation() {
        let current = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-current",
            200,
            2_000,
        );
        let stale = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-stale",
            201,
            1_999,
        );
        let kenya = entry("kenya", None, RegionLevel::Country, "cid-kenya", 100, 2_500);

        let mut registry = Registry::default();
        registry.register_batch(vec![current]).unwrap();

        let before = registry.clone();
        let err = registry.register_batch(vec![kenya, stale]).unwrap_err();

        assert!(matches!(err, RegistryError::StaleTimestamp { .. }));
        assert_eq!(registry, before);
    }

    #[test]
    fn version_downgrade_is_rejected_even_with_newer_registration_timestamp() {
        let current = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-current",
            200,
            2_000,
        );
        let downgrade = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-downgrade",
            199,
            3_000,
        );

        let mut registry = Registry::default();
        registry.register_batch(vec![current]).unwrap();

        assert!(matches!(
            registry.register_batch(vec![downgrade]),
            Err(RegistryError::VersionDowngrade { .. })
        ));
    }

    #[test]
    fn duplicate_region_inside_one_batch_is_rejected() {
        let a = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-a",
            100,
            1_000,
        );
        let b = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid-b",
            101,
            2_000,
        );

        let mut registry = Registry::default();
        assert!(matches!(
            registry.register_batch(vec![a, b]),
            Err(RegistryError::DuplicateRegionInBatch { .. })
        ));
        assert!(registry.entries.is_empty());
    }

    #[test]
    fn subregion_requires_parent_and_country_rejects_parent() {
        let bad_country = entry(
            "switzerland",
            Some("europe"),
            RegionLevel::Country,
            "zDv-ch",
            1,
            1,
        );
        assert!(matches!(
            bad_country.validate(),
            Err(RegistryError::InvalidParent { .. })
        ));

        let bad_subregion = entry(
            "us/california",
            None,
            RegionLevel::Subregion,
            "zDv-ca",
            1,
            1,
        );
        assert!(matches!(
            bad_subregion.validate(),
            Err(RegistryError::InvalidParent { .. })
        ));
    }

    #[test]
    fn stored_entries_and_parent_queries_are_timestamp_ordered() {
        let mut registry = Registry::default();
        registry
            .register_batch(vec![
                entry(
                    "us/california",
                    Some("us"),
                    RegionLevel::Subregion,
                    "cid-ca",
                    1,
                    10,
                ),
                entry(
                    "us/texas",
                    Some("us"),
                    RegionLevel::Subregion,
                    "cid-tx",
                    1,
                    30,
                ),
                entry(
                    "india/northern-zone",
                    Some("india"),
                    RegionLevel::Subregion,
                    "cid-in",
                    1,
                    20,
                ),
            ])
            .unwrap();

        assert_eq!(registry.entries[0].region, "us/texas");
        assert_eq!(registry.entries[1].region, "india/northern-zone");
        assert_eq!(registry.entries[2].region, "us/california");

        let us = registry.by_parent("us");
        assert_eq!(us.len(), 2);
        assert_eq!(us[0].region, "us/texas");
        assert_eq!(us[1].region, "us/california");
    }

    #[test]
    fn cid_query_can_return_multiple_current_regions_and_preserves_order() {
        let mut registry = Registry::default();
        registry
            .register_batch(vec![
                entry(
                    "us/texas",
                    Some("us"),
                    RegionLevel::Subregion,
                    "shared-cid",
                    1,
                    20,
                ),
                entry(
                    "india/northern-zone",
                    Some("india"),
                    RegionLevel::Subregion,
                    "shared-cid",
                    1,
                    30,
                ),
            ])
            .unwrap();

        let shared = registry.by_cid("shared-cid");
        assert_eq!(shared.len(), 2);
        assert_eq!(shared[0].region, "india/northern-zone");
        assert_eq!(shared[1].region, "us/texas");
    }

    #[test]
    fn changing_region_shape_is_rejected() {
        let current = entry(
            "us/california",
            Some("us"),
            RegionLevel::Subregion,
            "cid-a",
            1,
            10,
        );
        let changed = entry(
            "us/california",
            None,
            RegionLevel::Country,
            "cid-b",
            2,
            20,
        );

        let mut registry = Registry::default();
        registry.register_batch(vec![current]).unwrap();

        assert!(matches!(
            registry.register_batch(vec![changed]),
            Err(RegistryError::RegionShapeChanged { .. })
        ));
    }

    #[test]
    fn checksum_must_be_md5_hex() {
        let mut bad = entry("ethiopia", None, RegionLevel::Country, "cid", 1, 1);
        bad.checksum = "not-md5".to_string();

        assert_eq!(bad.validate(), Err(RegistryError::InvalidChecksum));
    }
}
