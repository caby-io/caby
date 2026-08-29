use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use jiff::{civil, tz::TimeZone, Timestamp};
use yaml_rust2::{Yaml, YamlLoader};

use crate::{files::CABY_SHARE_SPEC_EXT, share::ShareLimits, user::Permission, Result};

pub fn spec_root(spec_path: &Path) -> Option<PathBuf> {
    let name = spec_path.file_name()?.to_str()?;
    let stem = name.strip_suffix(CABY_SHARE_SPEC_EXT)?.strip_suffix('.')?;
    if stem.is_empty() {
        return None;
    }
    Some(spec_path.with_file_name(stem))
}

pub struct ShareSpec {
    pub account_flows: Vec<SpecFlow>,
    pub guest_flows: Vec<SpecFlow>,
    pub expires_at: Option<Timestamp>,
}

pub struct SpecFlow {
    pub password: Option<String>,
    pub permissions: BTreeSet<Permission>,
    pub limits: Option<ShareLimits>,
}

impl ShareSpec {
    pub fn try_parse(content: &str) -> Result<Self> {
        let docs =
            YamlLoader::load_from_str(content).context("could not parse share spec as yaml")?;
        let doc = docs.first().ok_or_else(|| anyhow!("share spec is empty"))?;
        Self::try_from(doc)
    }
}

impl TryFrom<&Yaml> for ShareSpec {
    type Error = crate::Error;

    fn try_from(doc: &Yaml) -> Result<Self> {
        let account_flows = parse_flows(&doc["account_flows"], "account_flows")?;
        let guest_flows = parse_flows(&doc["guest_flows"], "guest_flows")?;

        let expires_at = match &doc["expires_at"] {
            Yaml::BadValue | Yaml::Null => None,
            Yaml::String(s) => Some(parse_expiry(s)?),
            _ => return Err(anyhow!(".expires_at must be a string")),
        };

        Ok(Self {
            account_flows,
            guest_flows,
            expires_at,
        })
    }
}

fn parse_flows(node: &Yaml, field: &str) -> Result<Vec<SpecFlow>> {
    let flows = match node {
        Yaml::BadValue | Yaml::Null => return Ok(vec![]),
        Yaml::Array(arr) => arr,
        _ => return Err(anyhow!(".{} must be an array", field)),
    };

    flows
        .iter()
        .enumerate()
        .map(|(i, flow)| parse_flow(flow, field, i))
        .collect()
}

fn parse_flow(flow: &Yaml, field: &str, i: usize) -> Result<SpecFlow> {
    let password = match &flow["password"] {
        Yaml::BadValue | Yaml::Null => None,
        Yaml::String(s) => Some(s.clone()),
        _ => return Err(anyhow!(".{}[{}].password must be a string", field, i)),
    };

    let permissions = parse_permissions(&flow["permissions"], field, i)?;
    if permissions.is_empty() {
        return Err(anyhow!(
            ".{}[{}].permissions must grant at least one permission",
            field,
            i
        ));
    }

    let limits = parse_limits(&flow["limits"], field, i)?;

    Ok(SpecFlow {
        password,
        permissions,
        limits,
    })
}

fn parse_permissions(node: &Yaml, field: &str, i: usize) -> Result<BTreeSet<Permission>> {
    let items = match node {
        Yaml::Array(arr) => arr,
        Yaml::BadValue | Yaml::Null => {
            return Err(anyhow!(".{}[{}].permissions is required", field, i))
        }
        _ => return Err(anyhow!(".{}[{}].permissions must be an array", field, i)),
    };

    let mut permissions = BTreeSet::new();
    for perm in items {
        let name = match perm {
            Yaml::String(s) => s.as_str(),
            _ => return Err(anyhow!(".{}[{}].permissions must be strings", field, i)),
        };
        let permission =
            Permission::try_from(name).with_context(|| format!(".{}[{}].permissions", field, i))?;
        permissions.insert(permission);
    }

    Ok(permissions)
}

fn parse_limits(node: &Yaml, field: &str, i: usize) -> Result<Option<ShareLimits>> {
    match node {
        Yaml::BadValue | Yaml::Null => Ok(None),
        Yaml::Hash(_) => Ok(Some(ShareLimits {
            max_file_bytes: parse_opt_u64(&node["max_file_bytes"], field, i, "max_file_bytes")?,
            max_bytes_per_day: parse_opt_u64(
                &node["max_bytes_per_day"],
                field,
                i,
                "max_bytes_per_day",
            )?,
            max_files_per_day: parse_opt_u64(
                &node["max_files_per_day"],
                field,
                i,
                "max_files_per_day",
            )?,
        })),
        _ => Err(anyhow!(".{}[{}].limits must be a mapping", field, i)),
    }
}

fn parse_expiry(s: &str) -> Result<Timestamp> {
    let s = s.trim();

    if let Ok(ts) = s.parse::<Timestamp>() {
        return Ok(ts);
    }

    let date_only = !s.contains('T') && !s.contains(' ');
    if date_only {
        if let Ok(date) = s.parse::<civil::Date>() {
            return Ok(date.tomorrow()?.to_zoned(TimeZone::UTC)?.timestamp());
        }
    } else if let Ok(dt) = s.parse::<civil::DateTime>() {
        return Ok(dt.to_zoned(TimeZone::UTC)?.timestamp());
    }

    Err(anyhow!(
        ".expires_at '{}' must be a date (2026-12-31) or timestamp (2026-12-31T23:59:00Z)",
        s
    ))
}

fn parse_opt_u64(node: &Yaml, field: &str, i: usize, key: &str) -> Result<Option<u64>> {
    match node {
        Yaml::BadValue | Yaml::Null => Ok(None),
        Yaml::Integer(n) if *n >= 0 => Ok(Some(*n as u64)),
        Yaml::Integer(_) => Err(anyhow!(
            ".{}[{}].limits.{} must be non-negative",
            field,
            i,
            key
        )),
        _ => Err(anyhow!(
            ".{}[{}].limits.{} must be an integer",
            field,
            i,
            key
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_full_spec() {
        let spec = ShareSpec::try_parse(
            r#"
guest_flows:
  - permissions: [view, download]
account_flows:
  - password: hunter2
    permissions: [view, download, write]
    limits:
      max_file_bytes: 1048576
      max_files_per_day: 10
expires_at: 2026-12-31T00:00:00Z
"#,
        )
        .unwrap();

        assert!(spec.expires_at.is_some());

        assert_eq!(spec.guest_flows.len(), 1);
        assert!(spec.guest_flows[0].password.is_none());
        assert_eq!(
            spec.guest_flows[0].permissions,
            BTreeSet::from([Permission::View, Permission::Download])
        );

        assert_eq!(spec.account_flows.len(), 1);
        assert_eq!(spec.account_flows[0].password.as_deref(), Some("hunter2"));
        let limits = spec.account_flows[0].limits.as_ref().unwrap();
        assert_eq!(limits.max_file_bytes, Some(1048576));
        assert_eq!(limits.max_bytes_per_day, None);
        assert_eq!(limits.max_files_per_day, Some(10));
    }

    #[test]
    fn omitted_flows_default_to_empty() {
        let spec = ShareSpec::try_parse("guest_flows:\n  - permissions: [view]").unwrap();
        assert_eq!(spec.guest_flows.len(), 1);
        assert!(spec.account_flows.is_empty());
        assert!(spec.expires_at.is_none());
    }

    #[test]
    fn rejects_empty_document() {
        assert!(ShareSpec::try_parse("").is_err());
    }

    #[test]
    fn rejects_unknown_permission() {
        assert!(ShareSpec::try_parse("guest_flows:\n  - permissions: [teleport]").is_err());
    }

    #[test]
    fn rejects_empty_permissions() {
        assert!(ShareSpec::try_parse("guest_flows:\n  - permissions: []").is_err());
    }

    #[test]
    fn rejects_invalid_expiry() {
        assert!(ShareSpec::try_parse("expires_at: not-a-date").is_err());
    }

    #[test]
    fn bare_date_expires_at_end_of_that_day_utc() {
        let spec = ShareSpec::try_parse("expires_at: 2026-12-31").unwrap();
        assert_eq!(
            spec.expires_at,
            Some("2027-01-01T00:00:00Z".parse().unwrap())
        );
    }

    #[test]
    fn datetime_without_offset_is_utc() {
        let spec = ShareSpec::try_parse("expires_at: 2026-12-31T09:30:00").unwrap();
        assert_eq!(
            spec.expires_at,
            Some("2026-12-31T09:30:00Z".parse().unwrap())
        );
    }

    #[test]
    fn full_timestamp_keeps_its_offset() {
        let spec = ShareSpec::try_parse("expires_at: 2026-12-31T09:30:00+02:00").unwrap();
        assert_eq!(
            spec.expires_at,
            Some("2026-12-31T07:30:00Z".parse().unwrap())
        );
    }

    #[test]
    fn spec_root_strips_the_sidecar_suffix() {
        assert_eq!(
            spec_root(Path::new("photos/trip.share.caby")),
            Some(PathBuf::from("photos/trip"))
        );
        assert_eq!(
            spec_root(Path::new("album.share.caby")),
            Some(PathBuf::from("album"))
        );
    }

    #[test]
    fn spec_root_rejects_a_bare_suffix() {
        assert_eq!(spec_root(Path::new(".share.caby")), None);
        assert_eq!(spec_root(Path::new("notashare.txt")), None);
    }

    #[test]
    fn rejects_negative_limit() {
        assert!(ShareSpec::try_parse(
            "guest_flows:\n  - permissions: [view]\n    limits:\n      max_file_bytes: -1"
        )
        .is_err());
    }
}
