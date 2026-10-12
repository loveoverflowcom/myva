//! Small explicit kit → load unit mapping. No inferred identity or cross-kit fallback.
use myva_assets::{AssetKind, Bundle, BundleKind, Manifest, Role};

use crate::art::ArtFixture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetReview {
    /// D05-A procedural art with provenance/license; does not approve canon artwork.
    PlaceholderFixture,
    Approved {
        decision: &'static str,
    },
    Unapproved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitAssetMapping {
    pub kit: &'static str,
    pub bundle: &'static str,
    pub sheet: &'static str,
    pub kind: BundleKind,
    pub review: AssetReview,
}

pub const KIT_ASSETS: &[KitAssetMapping] = &[
    KitAssetMapping {
        kit: "long-luu",
        bundle: "character.long_luu",
        sheet: "long_luu.sheet",
        kind: BundleKind::Character,
        review: AssetReview::PlaceholderFixture,
    },
    KitAssetMapping {
        kit: "ke-giu-dap",
        bundle: "boss.ke_giu_dap",
        sheet: "ke_giu_dap.sheet",
        kind: BundleKind::Boss,
        review: AssetReview::PlaceholderFixture,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MappingError {
    UnknownKit(String),
    DuplicateKit(String),
    Unapproved(String),
    MissingBundle(String),
    InvalidBundle(String),
    MissingSheet(String),
    SheetIdentity(String),
    UnsupportedDependencies(String),
    Planner(String),
}

impl std::fmt::Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "asset mapping: {self:?}")
    }
}
impl std::error::Error for MappingError {}

pub fn resolve<'a>(
    manifest: &'a Manifest,
    mappings: &'a [KitAssetMapping],
    kit: &str,
) -> Result<(&'a KitAssetMapping, &'a Bundle), MappingError> {
    let mut matches = mappings.iter().filter(|m| m.kit == kit);
    let mapping = matches
        .next()
        .ok_or_else(|| MappingError::UnknownKit(kit.to_owned()))?;
    if matches.next().is_some() {
        return Err(MappingError::DuplicateKit(kit.to_owned()));
    }
    if matches!(mapping.review, AssetReview::Unapproved)
        || matches!(mapping.review, AssetReview::Approved { decision } if decision.trim().is_empty())
    {
        return Err(MappingError::Unapproved(kit.to_owned()));
    }
    let bundle = bundle(manifest, mapping.bundle, mapping.kind)?;
    if !bundle.assets.iter().any(|id| id == mapping.sheet)
        || manifest
            .asset(mapping.sheet)
            .is_none_or(|a| a.kind != AssetKind::SpriteSheet)
    {
        return Err(MappingError::MissingSheet(mapping.sheet.to_owned()));
    }
    Ok((mapping, bundle))
}

/// The current bounded runtime supports self-contained units only, explicitly rejecting deps.
pub(crate) fn bundle<'a>(
    manifest: &'a Manifest,
    id: &str,
    kind: BundleKind,
) -> Result<&'a Bundle, MappingError> {
    let bundle = manifest
        .bundles
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| MappingError::MissingBundle(id.to_owned()))?;
    if bundle.kind != kind {
        return Err(MappingError::InvalidBundle(id.to_owned()));
    }
    if !bundle.dependencies.is_empty() {
        return Err(MappingError::UnsupportedDependencies(id.to_owned()));
    }
    Ok(bundle)
}

/// Validated manifest bytes alone cannot approve a sprite's subject. Check the loaded document
/// before it can own render handles or cover a fighter's geometric fallback.
pub(crate) fn verify_fixture(
    fixture: &ArtFixture,
    mapping: &KitAssetMapping,
) -> Result<(), MappingError> {
    let sheet = fixture
        .documents
        .sheets
        .get(mapping.sheet)
        .ok_or_else(|| MappingError::MissingSheet(mapping.sheet.to_owned()))?;
    let role = match mapping.kind {
        BundleKind::Character => Role::Character,
        BundleKind::Boss => Role::Boss,
        _ => return Err(MappingError::InvalidBundle(mapping.bundle.to_owned())),
    };
    if sheet.subject.as_deref() != Some(mapping.kit)
        || sheet.role != role
        || fixture.documents.sheets.values().any(|s| {
            matches!(s.role, Role::Character | Role::Boss)
                && (s.subject.as_deref() != Some(mapping.kit) || s != sheet)
        })
    {
        return Err(MappingError::SheetIdentity(mapping.kit.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest() -> Manifest {
        myva_assets::parse_manifest(include_bytes!("../../../assets/fixture/manifest.json"))
            .unwrap()
    }

    #[test]
    fn mapping_rejects_unknown_duplicate_unapproved_and_missing_assets() {
        let mut manifest = manifest();
        assert!(matches!(
            resolve(&manifest, KIT_ASSETS, "unmapped"),
            Err(MappingError::UnknownKit(_))
        ));
        let duplicates = [KIT_ASSETS[0], KIT_ASSETS[0]];
        assert!(matches!(
            resolve(&manifest, &duplicates, "long-luu"),
            Err(MappingError::DuplicateKit(_))
        ));
        let unapproved = [KitAssetMapping {
            review: AssetReview::Unapproved,
            ..KIT_ASSETS[0]
        }];
        assert!(matches!(
            resolve(&manifest, &unapproved, "long-luu"),
            Err(MappingError::Unapproved(_))
        ));
        manifest.assets.retain(|a| a.id != "long_luu.sheet");
        assert!(matches!(
            resolve(&manifest, KIT_ASSETS, "long-luu"),
            Err(MappingError::MissingSheet(_))
        ));
        manifest.bundles.retain(|b| b.id != "character.long_luu");
        assert!(matches!(
            resolve(&manifest, KIT_ASSETS, "long-luu"),
            Err(MappingError::MissingBundle(_))
        ));
    }

    #[test]
    fn loaded_sheet_cannot_impersonate_another_kit() {
        let mut fixture = ArtFixture {
            manifest: manifest(),
            documents: Default::default(),
            textures: Default::default(),
        };
        let sheet = myva_assets::parse_sheet(
            "long_luu.sheet",
            include_bytes!("../../../assets/fixture/long_luu/sheet.json"),
        )
        .unwrap();
        fixture
            .documents
            .sheets
            .insert("long_luu.sheet".into(), sheet);
        assert_eq!(verify_fixture(&fixture, &KIT_ASSETS[0]), Ok(()));
        fixture
            .documents
            .sheets
            .get_mut("long_luu.sheet")
            .unwrap()
            .subject = Some("ke-giu-dap".into());
        assert!(matches!(
            verify_fixture(&fixture, &KIT_ASSETS[0]),
            Err(MappingError::SheetIdentity(_))
        ));
    }
}
