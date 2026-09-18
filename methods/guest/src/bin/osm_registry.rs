#![cfg_attr(not(test), no_main)]

use nssa_core::account::Data;
use osm_registry_core::{RegionLevel, Registry, RegistryEntry, RegistryError};
use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

const E_ARITY_MISMATCH: u32 = 100;
const E_REGISTRY_BYTES: u32 = 101;

fn map_registry_error(err: RegistryError) -> SpelError {
    SpelError::custom(1_000 + err.code(), err.to_string())
}

fn load_registry(registry: &AccountWithMetadata) -> Result<Registry, SpelError> {
    if registry.account.data.is_empty() {
        Ok(Registry::default())
    } else {
        borsh::from_slice::<Registry>(&registry.account.data)
            .map_err(|e| SpelError::SerializationError {
                message: e.to_string(),
            })
    }
}

fn save_registry(registry: &mut AccountWithMetadata, state: &Registry) -> Result<(), SpelError> {
    let bytes = borsh::to_vec(state).map_err(|e| SpelError::SerializationError {
        message: e.to_string(),
    })?;
    registry.account.data = Data::try_from(bytes).map_err(|_| {
        SpelError::custom(
            E_REGISTRY_BYTES,
            "registry state exceeds LEZ account-data capacity".to_string(),
        )
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_entries(
    regions: Vec<String>,
    parents: Vec<String>,
    levels: Vec<u8>,
    cids: Vec<String>,
    source_urls: Vec<String>,
    checksums: Vec<String>,
    versions: Vec<u64>,
    hosted: Vec<bool>,
    timestamps: Vec<u64>,
) -> Result<Vec<RegistryEntry>, SpelError> {
    let n = regions.len();
    let lengths = [
        ("parents", parents.len()),
        ("levels", levels.len()),
        ("cids", cids.len()),
        ("source_urls", source_urls.len()),
        ("checksums", checksums.len()),
        ("versions", versions.len()),
        ("hosted", hosted.len()),
        ("timestamps", timestamps.len()),
    ];

    if let Some((name, actual)) = lengths.into_iter().find(|(_, len)| *len != n) {
        return Err(SpelError::custom(
            E_ARITY_MISMATCH,
            format!(
                "parallel batch vectors must have equal length: regions={n}, {name}={actual}"
            ),
        ));
    }

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let level = RegionLevel::try_from(levels[i]).map_err(map_registry_error)?;
        let parent = if parents[i].trim().is_empty() {
            None
        } else {
            Some(parents[i].clone())
        };

        out.push(RegistryEntry {
            region: regions[i].clone(),
            parent,
            level,
            cid: cids[i].clone(),
            source_url: source_urls[i].clone(),
            checksum: checksums[i].clone(),
            version: versions[i],
            hosted: hosted[i],
            timestamp: timestamps[i],
        });
    }

    Ok(out)
}

#[lez_program]
mod osm_registry {
    #[allow(unused_imports)]
    use super::*;

    /// Initialize the single registry PDA. Anyone may create it once.
    #[instruction]
    pub fn init_registry(
        #[account(init, pda = literal("osm-registry"))]
        mut registry: AccountWithMetadata,
        #[account(signer)]
        registrar: AccountWithMetadata,
    ) -> SpelResult {
        save_registry(&mut registry, &Registry::default())?;
        Ok(SpelOutput::execute(vec![registry, registrar], vec![]))
    }

    /// Register one or more OSM distribution entries in one transaction.
    /// Parallel vectors are matched by index; parent is empty for countries.
    /// Level values are 0=country and 1=subregion.
    #[instruction]
    #[allow(clippy::too_many_arguments)]
    pub fn register_batch(
        #[account(mut, pda = literal("osm-registry"))]
        mut registry: AccountWithMetadata,
        #[account(signer)]
        registrar: AccountWithMetadata,
        regions: Vec<String>,
        parents: Vec<String>,
        levels: Vec<u8>,
        cids: Vec<String>,
        source_urls: Vec<String>,
        checksums: Vec<String>,
        versions: Vec<u64>,
        hosted: Vec<bool>,
        timestamps: Vec<u64>,
    ) -> SpelResult {
        let entries = build_entries(
            regions,
            parents,
            levels,
            cids,
            source_urls,
            checksums,
            versions,
            hosted,
            timestamps,
        )?;

        let mut state = load_registry(&registry)?;
        state.register_batch(entries).map_err(map_registry_error)?;
        save_registry(&mut registry, &state)?;

        Ok(SpelOutput::execute(vec![registry, registrar], vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_builder_normalizes_empty_parent() {
        let entries = build_entries(
            vec!["ethiopia".into()],
            vec!["".into()],
            vec![0],
            vec!["cid".into()],
            vec!["https://download.geofabrik.de/africa/ethiopia-latest.osm.pbf".into()],
            vec!["426bd510159627dc139d4d0ad3bc6acd".into()],
            vec![1],
            vec![true],
            vec![2],
        )
        .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].parent, None);
        assert_eq!(entries[0].level, RegionLevel::Country);
    }

    #[test]
    fn batch_builder_rejects_mismatched_parallel_vectors() {
        let err = build_entries(
            vec!["ethiopia".into()],
            vec![],
            vec![0],
            vec!["cid".into()],
            vec!["https://example.invalid/x.osm.pbf".into()],
            vec!["426bd510159627dc139d4d0ad3bc6acd".into()],
            vec![1],
            vec![true],
            vec![2],
        )
        .unwrap_err();

        assert!(err.to_string().contains("parallel batch vectors"));
    }
}
