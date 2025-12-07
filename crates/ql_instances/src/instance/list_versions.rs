use ql_core::{json::Manifest, JsonDownloadError, ListEntry, ListEntryKind};

/// Returns a list of every downloadable version of Minecraft.
/// Sources the list from Mojang and Omniarchive (combined).
///
/// # Errors
/// - If the version [manifest](https://launchermeta.mojang.com/mc/game/version_manifest.json)
///   couldn't be downloaded
/// - If the version manifest couldn't be parsed into JSON
///
/// Note: If Omniarchive list download for old versions fails,
/// an error will be logged but not returned (for smoother user experience),
/// and instead the official (inferior) old version list will be downloaded
/// from Mojang.
pub async fn list_versions() -> Result<(Vec<ListEntry>, String), JsonDownloadError> {
    let manifest = Manifest::download().await?;
    let latest = manifest.get_latest_release().unwrap().id.clone();

    Ok((
        manifest
            .versions
            .into_iter()
            .map(|n| ListEntry {
                kind: ListEntryKind::calculate(&n.id, &n.r#type),
                supports_server: n.supports_server(),
                name: n.id,
            })
            .collect(),
        latest,
    ))
}
