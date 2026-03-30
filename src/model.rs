use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Safe,
    Caution,
    Advanced,
}

impl Risk {
    pub fn sort_key(self) -> u8 {
        match self {
            Risk::Safe => 0,
            Risk::Caution => 1,
            Risk::Advanced => 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageEntry {
    pub id: String,
    pub category: String,
    pub description: String,
    pub risk: Risk,
}

#[derive(Debug, Clone)]
pub struct InstalledBloat {
    pub meta: PackageEntry,
    pub selected: bool,
}

impl Ord for InstalledBloat {
    fn cmp(&self, other: &Self) -> Ordering {
        let r = self.meta.risk.sort_key().cmp(&other.meta.risk.sort_key());
        if r != Ordering::Equal {
            return r;
        }
        let c = self
            .meta
            .category
            .to_lowercase()
            .cmp(&other.meta.category.to_lowercase());
        if c != Ordering::Equal {
            return c;
        }
        self.meta.id.cmp(&other.meta.id)
    }
}

impl PartialOrd for InstalledBloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InstalledBloat {
    fn eq(&self, other: &Self) -> bool {
        self.meta.id == other.meta.id
    }
}

impl Eq for InstalledBloat {}

pub fn load_catalog() -> anyhow::Result<Vec<PackageEntry>> {
    const JSON: &str = include_str!("../data/packages.json");
    let entries: Vec<PackageEntry> = serde_json::from_str(JSON)?;
    Ok(entries)
}
