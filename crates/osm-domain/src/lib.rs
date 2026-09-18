use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionLevel {
    Country,
    Subregion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionDefinition {
    /// LP-0018 canonical leaf id. Country ids are bare; decomposed leaves use parent/id.
    pub id: &'static str,
    /// Geofabrik feature `properties.id`.
    pub source_id: &'static str,
    /// LP-0018 registry parent. Country-level leaves deliberately use `None`.
    pub parent: Option<&'static str>,
    /// Geofabrik feature `properties.parent`; kept separate from LP-0018 parent semantics.
    pub source_parent: &'static str,
    pub level: RegionLevel,
}

impl RegionDefinition {
    pub const fn country(id: &'static str, source_parent: &'static str) -> Self {
        Self {
            id,
            source_id: id,
            parent: None,
            source_parent,
            level: RegionLevel::Country,
        }
    }

    pub const fn subregion(
        id: &'static str,
        source_id: &'static str,
        parent: &'static str,
    ) -> Self {
        Self::subregion_source(id, source_id, parent, parent)
    }

    pub const fn subregion_source(
        id: &'static str,
        source_id: &'static str,
        parent: &'static str,
        source_parent: &'static str,
    ) -> Self {
        Self {
            id,
            source_id,
            parent: Some(parent),
            source_parent,
            level: RegionLevel::Subregion,
        }
    }
}

pub const PREDEFINED_REGIONS: [RegionDefinition; 72] = [
    // Europe — country-level leaves.
    RegionDefinition::country("germany", "europe"),
    RegionDefinition::country("france", "europe"),
    RegionDefinition::country("great-britain", "europe"),
    RegionDefinition::country("italy", "europe"),
    RegionDefinition::country("spain", "europe"),
    RegionDefinition::country("poland", "europe"),
    RegionDefinition::country("netherlands", "europe"),
    RegionDefinition::country("belgium", "europe"),
    RegionDefinition::country("switzerland", "europe"),
    RegionDefinition::country("austria", "europe"),
    RegionDefinition::country("czech-republic", "europe"),
    RegionDefinition::country("sweden", "europe"),
    RegionDefinition::country("norway", "europe"),
    RegionDefinition::country("denmark", "europe"),
    RegionDefinition::country("finland", "europe"),
    RegionDefinition::country("portugal", "europe"),
    RegionDefinition::country("greece", "europe"),
    // Geofabrik publishes the Ireland coverage extract under this path.
    RegionDefinition::country("ireland-and-northern-ireland", "europe"),
    RegionDefinition::country("hungary", "europe"),
    RegionDefinition::country("romania", "europe"),
    RegionDefinition::country("bulgaria", "europe"),
    RegionDefinition::country("ukraine", "europe"),
    RegionDefinition::country("belarus", "europe"),
    RegionDefinition::country("turkey", "europe"),
    // North America.
    RegionDefinition::country("canada", "north-america"),
    RegionDefinition::country("mexico", "north-america"),
    // Asia.
    RegionDefinition::country("japan", "asia"),
    RegionDefinition::country("south-korea", "asia"),
    RegionDefinition::country("indonesia", "asia"),
    RegionDefinition::country("thailand", "asia"),
    RegionDefinition::country("vietnam", "asia"),
    // Geofabrik publishes Malaysia only as the combined Malaysia/Singapore/Brunei extract.
    RegionDefinition::country("malaysia-singapore-brunei", "asia"),
    RegionDefinition::country("philippines", "asia"),
    RegionDefinition::country("pakistan", "asia"),
    RegionDefinition::country("bangladesh", "asia"),
    RegionDefinition::country("iran", "asia"),
    // Oceania.
    RegionDefinition::country("australia", "australia-oceania"),
    // South America.
    RegionDefinition::country("brazil", "south-america"),
    RegionDefinition::country("argentina", "south-america"),
    RegionDefinition::country("colombia", "south-america"),
    RegionDefinition::country("peru", "south-america"),
    RegionDefinition::country("chile", "south-america"),
    // Africa.
    RegionDefinition::country("south-africa", "africa"),
    RegionDefinition::country("egypt", "africa"),
    RegionDefinition::country("nigeria", "africa"),
    RegionDefinition::country("kenya", "africa"),
    RegionDefinition::country("morocco", "africa"),
    RegionDefinition::country("ethiopia", "africa"),
    // United States — decomposed into eight state leaves.
    RegionDefinition::subregion_source("us/california", "us/california", "us", "north-america"),
    RegionDefinition::subregion_source("us/texas", "us/texas", "us", "north-america"),
    RegionDefinition::subregion_source("us/florida", "us/florida", "us", "north-america"),
    RegionDefinition::subregion_source("us/new-york", "us/new-york", "us", "north-america"),
    RegionDefinition::subregion_source("us/washington", "us/washington", "us", "north-america"),
    RegionDefinition::subregion_source("us/illinois", "us/illinois", "us", "north-america"),
    RegionDefinition::subregion_source("us/georgia", "us/georgia", "us", "north-america"),
    RegionDefinition::subregion_source("us/pennsylvania", "us/pennsylvania", "us", "north-america"),
    // India — six zone leaves.
    RegionDefinition::subregion("india/central-zone", "central-zone", "india"),
    RegionDefinition::subregion("india/eastern-zone", "eastern-zone", "india"),
    RegionDefinition::subregion("india/north-eastern-zone", "north-eastern-zone", "india"),
    RegionDefinition::subregion("india/northern-zone", "northern-zone", "india"),
    RegionDefinition::subregion("india/southern-zone", "southern-zone", "india"),
    RegionDefinition::subregion("india/western-zone", "western-zone", "india"),
    // China — six province leaves.
    RegionDefinition::subregion("china/guangdong", "guangdong", "china"),
    RegionDefinition::subregion("china/jiangsu", "jiangsu", "china"),
    RegionDefinition::subregion("china/shandong", "shandong", "china"),
    RegionDefinition::subregion("china/zhejiang", "zhejiang", "china"),
    RegionDefinition::subregion("china/sichuan", "sichuan", "china"),
    RegionDefinition::subregion("china/henan", "henan", "china"),
    // Russia — four federal-district leaves.
    RegionDefinition::subregion(
        "russia/central-fed-district",
        "central-fed-district",
        "russia",
    ),
    RegionDefinition::subregion(
        "russia/northwestern-fed-district",
        "northwestern-fed-district",
        "russia",
    ),
    RegionDefinition::subregion("russia/volga-fed-district", "volga-fed-district", "russia"),
    RegionDefinition::subregion(
        "russia/siberian-fed-district",
        "siberian-fed-district",
        "russia",
    ),
];

pub fn predefined_region(id: &str) -> Option<&'static RegionDefinition> {
    PREDEFINED_REGIONS.iter().find(|region| region.id == id)
}

pub fn validate_predefined_regions() -> Result<(), String> {
    let mut canonical = HashSet::with_capacity(PREDEFINED_REGIONS.len());
    let mut source_selectors = HashSet::with_capacity(PREDEFINED_REGIONS.len());

    for region in &PREDEFINED_REGIONS {
        if !canonical.insert(region.id) {
            return Err(format!("duplicate canonical region id: {}", region.id));
        }

        if !source_selectors.insert((region.source_id, region.source_parent)) {
            return Err(format!(
                "duplicate Geofabrik selector: ({}, {})",
                region.source_id, region.source_parent
            ));
        }

        match region.level {
            RegionLevel::Country => {
                if region.parent.is_some() {
                    return Err(format!(
                        "country leaf {} must have no registry parent",
                        region.id
                    ));
                }
            }
            RegionLevel::Subregion => {
                let parent = region.parent.ok_or_else(|| {
                    format!("subregion {} is missing a registry parent", region.id)
                })?;
                let expected_prefix = format!("{parent}/");
                if !region.id.starts_with(&expected_prefix)
                    || region.id.len() == expected_prefix.len()
                {
                    return Err(format!(
                        "subregion id {} must be namespaced under registry parent {}",
                        region.id, parent
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_set_has_exactly_72_unique_leaves() {
        assert_eq!(PREDEFINED_REGIONS.len(), 72);
        validate_predefined_regions().unwrap();
    }

    #[test]
    fn canonical_parent_differs_from_geofabrik_parent_for_country_leaves() {
        let switzerland = predefined_region("switzerland").unwrap();
        assert_eq!(switzerland.parent, None);
        assert_eq!(switzerland.source_parent, "europe");
        assert_eq!(switzerland.level, RegionLevel::Country);
    }

    #[test]
    fn ireland_uses_current_geofabrik_leaf_path() {
        let ireland = predefined_region("ireland-and-northern-ireland").unwrap();
        assert_eq!(ireland.source_id, "ireland-and-northern-ireland");
        assert_eq!(ireland.source_parent, "europe");
        assert_eq!(ireland.parent, None);
        assert_eq!(ireland.level, RegionLevel::Country);
    }

    #[test]
    fn malaysia_uses_current_combined_geofabrik_leaf_path() {
        let malaysia = predefined_region("malaysia-singapore-brunei").unwrap();
        assert_eq!(malaysia.source_id, "malaysia-singapore-brunei");
        assert_eq!(malaysia.source_parent, "asia");
        assert_eq!(malaysia.parent, None);
        assert_eq!(malaysia.level, RegionLevel::Country);
    }

    #[test]
    fn source_selector_disambiguates_us_georgia() {
        let georgia = predefined_region("us/georgia").unwrap();
        assert_eq!(georgia.source_id, "us/georgia");
        assert_eq!(georgia.source_parent, "north-america");
        assert_eq!(georgia.parent, Some("us"));
    }
}
