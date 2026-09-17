use md5::{Digest, Md5};
use osm_domain::{predefined_region, validate_predefined_regions, RegionLevel, PREDEFINED_REGIONS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeofabrikError {
    #[error("failed to parse Geofabrik index JSON: {0}")]
    InvalidIndexJson(#[from] serde_json::Error),

    #[error("duplicate Geofabrik feature selector: id={id}, parent={parent:?}")]
    DuplicateSourceFeature { id: String, parent: Option<String> },

    #[error("unknown LP-0018 predefined region: {0}")]
    UnknownPredefinedRegion(String),

    #[error(
        "LP-0018 region {canonical_id} is missing from Geofabrik index (source id={source_id}, parent={source_parent})"
    )]
    MissingSourceRegion {
        canonical_id: String,
        source_id: String,
        source_parent: String,
    },

    #[error("Geofabrik feature {id} has no PBF URL")]
    MissingPbfUrl { id: String },

    #[error("invalid Geofabrik MD5 manifest: {0}")]
    InvalidMd5Manifest(String),

    #[error("checksum mismatch: expected {expected}, observed {observed}")]
    ChecksumMismatch { expected: String, observed: String },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid frozen LP-0018 region set: {0}")]
    InvalidFrozenSet(String),
}

#[derive(Debug, Deserialize)]
struct FeatureCollection {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    properties: Properties,
}

#[derive(Debug, Clone, Deserialize)]
struct Properties {
    id: String,
    parent: Option<String>,
    name: String,
    urls: Urls,
}

#[derive(Debug, Clone, Deserialize)]
struct Urls {
    pbf: Option<String>,
}

#[derive(Debug)]
pub struct GeofabrikIndex {
    features: HashMap<(String, Option<String>), Properties>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedRegion {
    /// LP-0018 canonical region id.
    pub id: String,
    /// Geofabrik feature id.
    pub source_id: String,
    /// LP-0018 registry parent (None for country-level leaves).
    pub parent: Option<String>,
    /// Geofabrik source parent used to disambiguate source feature ids.
    pub source_parent: String,
    pub level: RegionLevel,
    pub name: String,
    pub pbf_url: String,
    pub checksum_url: String,
}

impl GeofabrikIndex {
    pub fn from_json(input: &str) -> Result<Self, GeofabrikError> {
        let collection: FeatureCollection = serde_json::from_str(input)?;
        let mut features = HashMap::with_capacity(collection.features.len());

        for feature in collection.features {
            let key = (
                feature.properties.id.clone(),
                feature.properties.parent.clone(),
            );
            if features.insert(key.clone(), feature.properties).is_some() {
                return Err(GeofabrikError::DuplicateSourceFeature {
                    id: key.0,
                    parent: key.1,
                });
            }
        }

        Ok(Self { features })
    }

    pub fn resolve(&self, canonical_id: &str) -> Result<ResolvedRegion, GeofabrikError> {
        let definition = predefined_region(canonical_id)
            .ok_or_else(|| GeofabrikError::UnknownPredefinedRegion(canonical_id.to_owned()))?;

        let key = (
            definition.source_id.to_owned(),
            Some(definition.source_parent.to_owned()),
        );
        let source =
            self.features
                .get(&key)
                .ok_or_else(|| GeofabrikError::MissingSourceRegion {
                    canonical_id: definition.id.to_owned(),
                    source_id: definition.source_id.to_owned(),
                    source_parent: definition.source_parent.to_owned(),
                })?;

        let pbf_url = source
            .urls
            .pbf
            .clone()
            .ok_or_else(|| GeofabrikError::MissingPbfUrl {
                id: definition.id.to_owned(),
            })?;

        Ok(ResolvedRegion {
            id: definition.id.to_owned(),
            source_id: definition.source_id.to_owned(),
            parent: definition.parent.map(str::to_owned),
            source_parent: definition.source_parent.to_owned(),
            level: definition.level,
            name: source.name.clone(),
            checksum_url: format!("{pbf_url}.md5"),
            pbf_url,
        })
    }

    pub fn resolve_all(&self) -> Result<Vec<ResolvedRegion>, GeofabrikError> {
        validate_predefined_regions().map_err(GeofabrikError::InvalidFrozenSet)?;
        PREDEFINED_REGIONS
            .iter()
            .map(|definition| self.resolve(definition.id))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Md5Manifest {
    pub expected_md5: String,
    pub filename: Option<String>,
}

pub fn parse_md5_manifest(input: &str) -> Result<Md5Manifest, GeofabrikError> {
    let mut parts = input.split_whitespace();
    let checksum = parts
        .next()
        .ok_or_else(|| GeofabrikError::InvalidMd5Manifest("manifest is empty".into()))?
        .to_ascii_lowercase();

    if checksum.len() != 32 || !checksum.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(GeofabrikError::InvalidMd5Manifest(format!(
            "expected 32 hex characters, got {checksum:?}"
        )));
    }

    let filename = parts
        .next()
        .map(|part| part.trim_start_matches('*').to_owned());

    Ok(Md5Manifest {
        expected_md5: checksum,
        filename,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HashResult {
    pub md5: String,
    pub byte_length: u64,
}

pub fn hash_reader<R: Read>(mut reader: R) -> Result<HashResult, io::Error> {
    let mut hasher = Md5::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut byte_length = 0_u64;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_length += read as u64;
    }

    Ok(HashResult {
        md5: format!("{:x}", hasher.finalize()),
        byte_length,
    })
}

pub fn hash_file(path: impl AsRef<Path>) -> Result<HashResult, GeofabrikError> {
    Ok(hash_reader(File::open(path)?)?)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    pub expected_md5: String,
    pub observed_md5: String,
    pub byte_length: u64,
    pub filename: Option<String>,
}

pub fn verify_reader<R: Read>(
    reader: R,
    manifest: &Md5Manifest,
) -> Result<VerificationReport, GeofabrikError> {
    let observed = hash_reader(reader)?;
    if observed.md5 != manifest.expected_md5 {
        return Err(GeofabrikError::ChecksumMismatch {
            expected: manifest.expected_md5.clone(),
            observed: observed.md5,
        });
    }

    Ok(VerificationReport {
        expected_md5: manifest.expected_md5.clone(),
        observed_md5: observed.md5,
        byte_length: observed.byte_length,
        filename: manifest.filename.clone(),
    })
}

pub fn verify_file(
    path: impl AsRef<Path>,
    manifest_text: &str,
) -> Result<VerificationReport, GeofabrikError> {
    let manifest = parse_md5_manifest(manifest_text)?;
    verify_reader(File::open(path)?, &manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use osm_domain::PREDEFINED_REGIONS;
    use serde_json::json;
    use std::io::Cursor;

    fn sample_index() -> String {
        json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {
                        "id": "switzerland",
                        "parent": "europe",
                        "name": "Switzerland",
                        "urls": { "pbf": "https://download.geofabrik.de/europe/switzerland-latest.osm.pbf" }
                    }
                },
                {
                    "type": "Feature",
                    "properties": {
                        "id": "georgia",
                        "parent": "europe",
                        "name": "Georgia",
                        "urls": { "pbf": "https://download.geofabrik.de/europe/georgia-latest.osm.pbf" }
                    }
                },
                {
                    "type": "Feature",
                    "properties": {
                        "id": "georgia",
                        "parent": "us",
                        "name": "Georgia",
                        "urls": { "pbf": "https://download.geofabrik.de/north-america/us/georgia-latest.osm.pbf" }
                    }
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn resolves_country_with_lp_parent_normalization() {
        let index = GeofabrikIndex::from_json(&sample_index()).unwrap();
        let region = index.resolve("switzerland").unwrap();
        assert_eq!(region.parent, None);
        assert_eq!(region.source_parent, "europe");
        assert_eq!(region.level, RegionLevel::Country);
        assert_eq!(
            region.checksum_url,
            "https://download.geofabrik.de/europe/switzerland-latest.osm.pbf.md5"
        );
    }

    #[test]
    fn source_parent_disambiguates_us_georgia_from_country_georgia() {
        let index = GeofabrikIndex::from_json(&sample_index()).unwrap();
        let region = index.resolve("us/georgia").unwrap();
        assert_eq!(region.parent.as_deref(), Some("us"));
        assert_eq!(region.source_parent, "us");
        assert!(region.pbf_url.contains("north-america/us/georgia"));
    }

    #[test]
    fn synthetic_index_covers_all_72_frozen_leaves() {
        let features: Vec<_> = PREDEFINED_REGIONS
            .iter()
            .map(|region| {
                json!({
                    "type": "Feature",
                    "properties": {
                        "id": region.source_id,
                        "parent": region.source_parent,
                        "name": region.id,
                        "urls": {
                            "pbf": format!("https://example.invalid/{}-latest.osm.pbf", region.source_id)
                        }
                    }
                })
            })
            .collect();
        let json = json!({ "type": "FeatureCollection", "features": features }).to_string();
        let index = GeofabrikIndex::from_json(&json).unwrap();
        let resolved = index.resolve_all().unwrap();
        assert_eq!(resolved.len(), 72);
    }

    #[test]
    fn parses_standard_md5_manifest() {
        let manifest =
            parse_md5_manifest("5d41402abc4b2a76b9719d911017c592  switzerland-latest.osm.pbf\n")
                .unwrap();
        assert_eq!(manifest.expected_md5, "5d41402abc4b2a76b9719d911017c592");
        assert_eq!(
            manifest.filename.as_deref(),
            Some("switzerland-latest.osm.pbf")
        );
    }

    #[test]
    fn verifies_streamed_bytes_without_loading_a_pbf_into_memory() {
        let manifest =
            parse_md5_manifest("5d41402abc4b2a76b9719d911017c592  sample.osm.pbf\n").unwrap();
        let report = verify_reader(Cursor::new(b"hello"), &manifest).unwrap();
        assert_eq!(report.observed_md5, manifest.expected_md5);
        assert_eq!(report.byte_length, 5);
    }

    #[test]
    fn checksum_mismatch_fails_closed() {
        let manifest =
            parse_md5_manifest("00000000000000000000000000000000  sample.osm.pbf\n").unwrap();
        let error = verify_reader(Cursor::new(b"hello"), &manifest).unwrap_err();
        match error {
            GeofabrikError::ChecksumMismatch { expected, observed } => {
                assert_eq!(expected, "00000000000000000000000000000000");
                assert_eq!(observed, "5d41402abc4b2a76b9719d911017c592");
            }
            other => panic!("expected checksum mismatch, got {other:?}"),
        }
    }
}
