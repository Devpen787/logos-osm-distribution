use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const REGISTRY_SCHEMA_VERSION: u8 = 1;
pub const MAX_BATCH: usize = 50;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
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

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
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
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct Registry {
    pub schema_version: u8,
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

        for entry in &entries {
            entry.validate()?;
        }

        // Work against a clone so a conflict anywhere in the batch cannot leave
        // a partially-mutated registry.
        let mut candidate = self.clone();
        let mut inserted = 0;
        let mut skipped_idempotent = 0;

        for entry in entries {
            if let Some(existing) = candidate
                .entries
                .iter()
                .find(|e| e.region == entry.region && e.cid == entry.cid)
            {
                if existing == &entry {
                    skipped_idempotent += 1;
                    continue;
                }
                return Err(RegistryError::ConflictingDuplicate {
                    region: entry.region,
                    cid: entry.cid,
                });
            }

            candidate.entries.push(entry);
            inserted += 1;
        }

        *self = candidate;
        Ok(RegistrationOutcome {
            inserted,
            skipped_idempotent,
        })
    }

    pub fn by_region(&self, region: &str) -> Vec<&RegistryEntry> {
        let mut out: Vec<_> = self.entries.iter().filter(|e| e.region == region).collect();
        sort_newest_first(&mut out);
        out
    }

    pub fn by_parent(&self, parent: &str) -> Vec<&RegistryEntry> {
        let mut out: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.parent.as_deref() == Some(parent))
            .collect();
        sort_newest_first(&mut out);
        out
    }

    pub fn by_cid(&self, cid: &str) -> Vec<&RegistryEntry> {
        let mut out: Vec<_> = self.entries.iter().filter(|e| e.cid == cid).collect();
        sort_newest_first(&mut out);
        out
    }

    pub fn latest_by_region(&self, region: &str) -> Option<&RegistryEntry> {
        self.by_region(region).into_iter().next()
    }
}

fn sort_newest_first(entries: &mut [&RegistryEntry]) {
    entries.sort_by(|a, b| {
        b.timestamp
            .cmp(&a.timestamp)
            .then_with(|| b.version.cmp(&a.version))
            .then_with(|| a.cid.cmp(&b.cid))
    });
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryError {
    #[error("batch is empty")]
    BatchEmpty,
    #[error("batch size {actual} exceeds max {max}")]
    BatchTooBig { actual: usize, max: usize },
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
    #[error("same region/CID already exists with different metadata: {region} / {cid}")]
    ConflictingDuplicate { region: String, cid: String },
}

impl RegistryError {
    pub const fn code(&self) -> u32 {
        match self {
            Self::BatchEmpty => 1,
            Self::BatchTooBig { .. } => 2,
            Self::InvalidRegion => 3,
            Self::InvalidParent { .. } => 4,
            Self::InvalidLevel(_) => 5,
            Self::InvalidCid => 6,
            Self::InvalidSourceUrl => 7,
            Self::InvalidChecksum => 8,
            Self::InvalidVersion => 9,
            Self::InvalidTimestamp => 10,
            Self::ConflictingDuplicate { .. } => 11,
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
    fn batch_registration_is_idempotent_for_identical_region_cid() {
        let ethiopia = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "zDv-ethiopia",
            1_789_685_417,
            1_789_700_000,
        );
        let mut registry = Registry::default();

        let first = registry.register_batch(vec![ethiopia.clone()]).unwrap();
        assert_eq!(first.inserted, 1);
        assert_eq!(first.skipped_idempotent, 0);

        let second = registry.register_batch(vec![ethiopia]).unwrap();
        assert_eq!(second.inserted, 0);
        assert_eq!(second.skipped_idempotent, 1);
        assert_eq!(registry.entries.len(), 1);
    }

    #[test]
    fn conflicting_duplicate_fails_without_partial_mutation() {
        let ethiopia = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "zDv-ethiopia",
            100,
            200,
        );
        let mut conflicting = ethiopia.clone();
        conflicting.version = 101;

        let kenya = entry(
            "kenya",
            None,
            RegionLevel::Country,
            "zDv-kenya",
            100,
            200,
        );

        let mut registry = Registry::default();
        registry.register_batch(vec![ethiopia]).unwrap();

        let before = registry.clone();
        let err = registry
            .register_batch(vec![kenya, conflicting])
            .unwrap_err();

        assert!(matches!(err, RegistryError::ConflictingDuplicate { .. }));
        assert_eq!(registry, before);
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
    fn region_query_orders_newest_timestamp_then_version() {
        let mut registry = Registry::default();
        registry
            .register_batch(vec![
                entry(
                    "ethiopia",
                    None,
                    RegionLevel::Country,
                    "cid-old",
                    100,
                    1000,
                ),
                entry(
                    "ethiopia",
                    None,
                    RegionLevel::Country,
                    "cid-new-low-version",
                    150,
                    2000,
                ),
                entry(
                    "ethiopia",
                    None,
                    RegionLevel::Country,
                    "cid-new-high-version",
                    200,
                    2000,
                ),
            ])
            .unwrap();

        let found = registry.by_region("ethiopia");
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].cid, "cid-new-high-version");
        assert_eq!(found[1].cid, "cid-new-low-version");
        assert_eq!(found[2].cid, "cid-old");
        assert_eq!(
            registry.latest_by_region("ethiopia").unwrap().cid,
            "cid-new-high-version"
        );
    }

    #[test]
    fn parent_and_cid_queries_are_supported() {
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

        let us = registry.by_parent("us");
        assert_eq!(us.len(), 2);
        assert_eq!(us[0].region, "us/texas");

        let shared = registry.by_cid("shared-cid");
        assert_eq!(shared.len(), 2);
        assert_eq!(shared[0].region, "india/northern-zone");
    }

    #[test]
    fn checksum_must_be_md5_hex() {
        let mut bad = entry(
            "ethiopia",
            None,
            RegionLevel::Country,
            "cid",
            1,
            1,
        );
        bad.checksum = "not-md5".to_string();

        assert_eq!(bad.validate(), Err(RegistryError::InvalidChecksum));
    }
}
