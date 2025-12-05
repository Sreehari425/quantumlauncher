//! LWJGL version fetching from Maven repository.
//!
//! This module provides functionality to fetch available LWJGL versions
//! from the Maven Central repository, allowing users to select custom
//! LWJGL versions for their Minecraft instances.

use crate::{file_utils, JsonDownloadError};

/// LWJGL 3.x Maven metadata URL
const LWJGL3_MAVEN_METADATA_URL: &str =
    "https://repo1.maven.org/maven2/org/lwjgl/lwjgl/maven-metadata.xml";

/// LWJGL 2.x Maven metadata URL (legacy)
const LWJGL2_MAVEN_METADATA_URL: &str =
    "https://repo1.maven.org/maven2/org/lwjgl/lwjgl/lwjgl/maven-metadata.xml";

/// A wrapper around a list of available LWJGL versions.
#[derive(Debug, Clone)]
pub struct LwjglVersionList {
    /// List of available versions, newest first (includes both 2.x and 3.x).
    pub versions: Vec<String>,
}

impl LwjglVersionList {
    /// Fetches the list of available LWJGL versions from Maven Central.
    /// Includes both LWJGL 2.x and 3.x versions.
    ///
    /// # Errors
    /// Returns an error if the network request fails or XML parsing fails.
    pub async fn download() -> Result<Self, JsonDownloadError> {
        // Fetch both LWJGL 2.x and 3.x versions in parallel
        let (lwjgl3_result, lwjgl2_result) = tokio::join!(
            file_utils::download_file_to_string(LWJGL3_MAVEN_METADATA_URL, false),
            file_utils::download_file_to_string(LWJGL2_MAVEN_METADATA_URL, false),
        );

        let mut versions = Vec::new();

        // Parse LWJGL 3.x versions
        if let Ok(xml) = lwjgl3_result {
            versions.extend(parse_maven_versions(&xml));
        }

        // Parse LWJGL 2.x versions
        if let Ok(xml) = lwjgl2_result {
            versions.extend(parse_maven_versions(&xml));
        }

        // Sort versions - newest first (simple version comparison)
        versions.sort_by(|a, b| compare_versions(b, a));

        // Remove duplicates while preserving order
        versions.dedup();

        Ok(Self { versions })
    }
}

/// Compare two version strings for sorting.
/// Returns ordering where higher versions come first.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split(|c: char| c == '.' || c == '-' || c == '+')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Parses Maven metadata XML to extract version strings.
///
/// The XML format is:
/// ```xml
/// <metadata>
///   <versioning>
///     <versions>
///       <version>3.0.0</version>
///       <version>3.1.0</version>
///       ...
///     </versions>
///   </versioning>
/// </metadata>
/// ```
fn parse_maven_versions(xml: &str) -> Vec<String> {
    let mut versions = Vec::new();

    // Simple XML parsing - find all <version>...</version> tags
    let mut remaining = xml;
    while let Some(start_idx) = remaining.find("<version>") {
        let after_tag = &remaining[start_idx + 9..]; // Skip "<version>"
        if let Some(end_idx) = after_tag.find("</version>") {
            let version = after_tag[..end_idx].trim().to_owned();
            if !version.is_empty() {
                versions.push(version);
            }
            remaining = &after_tag[end_idx + 10..]; // Skip "</version>"
        } else {
            break;
        }
    }

    // Reverse to get newest first (Maven lists oldest first)
    versions.reverse();
    versions
}

/// A wrapper type for displaying LWJGL versions in a pick_list.
/// `None` represents "Default (from game)".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LwjglVersion(pub Option<String>);

impl std::fmt::Display for LwjglVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(version) => write!(f, "{version}"),
            None => write!(f, "Default (from game)"),
        }
    }
}

impl LwjglVersion {
    /// Creates a list of `LwjglVersion` items for use in a pick_list,
    /// with "Default (from game)" as the first option.
    #[must_use]
    pub fn create_pick_list(versions: &[String]) -> Vec<Self> {
        let mut list = vec![Self(None)];
        list.extend(versions.iter().map(|v| Self(Some(v.clone()))));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_maven_versions() {
        let xml = r#"
            <metadata>
                <groupId>org.lwjgl</groupId>
                <artifactId>lwjgl</artifactId>
                <versioning>
                    <latest>3.3.6</latest>
                    <release>3.3.6</release>
                    <versions>
                        <version>3.0.0</version>
                        <version>3.1.0</version>
                        <version>3.2.0</version>
                        <version>3.3.6</version>
                    </versions>
                </versioning>
            </metadata>
        "#;

        let versions = parse_maven_versions(xml);
        assert_eq!(versions, vec!["3.3.6", "3.2.0", "3.1.0", "3.0.0"]);
    }

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;

        // 3.x > 2.x
        assert_eq!(compare_versions("3.3.6", "2.9.4"), Ordering::Greater);
        assert_eq!(compare_versions("2.9.4", "3.3.6"), Ordering::Less);

        // Same major, different minor
        assert_eq!(compare_versions("3.3.6", "3.2.0"), Ordering::Greater);
        assert_eq!(compare_versions("3.2.0", "3.3.6"), Ordering::Less);

        // Same version
        assert_eq!(compare_versions("3.3.6", "3.3.6"), Ordering::Equal);

        // With suffix
        assert_eq!(compare_versions("3.3.6", "3.3.5+1"), Ordering::Greater);
    }
}
