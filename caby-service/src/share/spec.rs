use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use jiff::{civil, tz::TimeZone, Timestamp};
use yaml_rust2::{yaml::Hash, Yaml, YamlEmitter, YamlLoader};

use crate::{files::CABY_SHARE_SPEC_EXT, share::ShareLimits, user::Permission, Result};

pub fn spec_root(spec_path: &Path) -> Option<PathBuf> {
    let name = spec_path.file_name()?.to_str()?;
    let stem = name.strip_suffix(CABY_SHARE_SPEC_EXT)?.strip_suffix('.')?;
    if stem.is_empty() {
        return None;
    }
    Some(
        spec_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default(),
    )
}

pub struct ShareSpec {
    pub account_flows: Vec<SpecFlow>,
    pub guest_flows: Vec<SpecFlow>,
    pub expires_at: Option<Timestamp>,
}

pub enum SpecAuth {
    Open,
    Password(String),
    Hash(String),
}

pub struct SpecFlow {
    pub auth: SpecAuth,
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

impl TryFrom<&ShareSpec> for String {
    type Error = crate::Error;

    fn try_from(spec: &ShareSpec) -> Result<Self> {
        let mut root = Hash::new();
        if !spec.account_flows.is_empty() {
            root.insert(
                yaml_str("account_flows"),
                flows_to_yaml(&spec.account_flows),
            );
        }
        if !spec.guest_flows.is_empty() {
            root.insert(yaml_str("guest_flows"), flows_to_yaml(&spec.guest_flows));
        }
        if let Some(expires_at) = spec.expires_at {
            root.insert(yaml_str("expires_at"), yaml_str(&expires_at.to_string()));
        }

        let mut out = String::new();
        YamlEmitter::new(&mut out)
            .dump(&Yaml::Hash(root))
            .map_err(|err| anyhow!("could not emit share spec: {}", err))?;
        Ok(out)
    }
}

fn yaml_str(s: &str) -> Yaml {
    Yaml::String(s.to_owned())
}

fn flows_to_yaml(flows: &[SpecFlow]) -> Yaml {
    Yaml::Array(flows.iter().map(flow_to_yaml).collect())
}

fn flow_to_yaml(flow: &SpecFlow) -> Yaml {
    let mut map = Hash::new();
    match &flow.auth {
        SpecAuth::Open => {}
        SpecAuth::Password(plaintext) => {
            map.insert(yaml_str("password"), yaml_str(plaintext));
        }
        SpecAuth::Hash(hash) => {
            map.insert(yaml_str("password_hash"), yaml_str(hash));
        }
    }
    let permissions = flow
        .permissions
        .iter()
        .map(|perm| yaml_str(<&str>::from(*perm)))
        .collect();
    map.insert(yaml_str("permissions"), Yaml::Array(permissions));
    if let Some(limits) = &flow.limits {
        map.insert(yaml_str("limits"), limits_to_yaml(limits));
    }
    Yaml::Hash(map)
}

fn limits_to_yaml(limits: &ShareLimits) -> Yaml {
    let mut map = Hash::new();
    if let Some(value) = limits.max_file_bytes {
        map.insert(yaml_str("max_file_bytes"), Yaml::Integer(value as i64));
    }
    if let Some(value) = limits.max_bytes_per_day {
        map.insert(yaml_str("max_bytes_per_day"), Yaml::Integer(value as i64));
    }
    if let Some(value) = limits.max_files_per_day {
        map.insert(yaml_str("max_files_per_day"), Yaml::Integer(value as i64));
    }
    Yaml::Hash(map)
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
    let auth = parse_auth(flow, field, i)?;

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
        auth,
        permissions,
        limits,
    })
}

fn parse_auth(flow: &Yaml, field: &str, i: usize) -> Result<SpecAuth> {
    let password = parse_opt_string(&flow["password"], field, i, "password")?;
    let password_hash = parse_opt_string(&flow["password_hash"], field, i, "password_hash")?;

    match (password, password_hash) {
        (Some(_), Some(_)) => Err(anyhow!(
            ".{}[{}] must not set both password and password_hash",
            field,
            i
        )),
        (Some(plaintext), None) => Ok(SpecAuth::Password(plaintext)),
        (None, Some(hash)) => Ok(SpecAuth::Hash(hash)),
        (None, None) => Ok(SpecAuth::Open),
    }
}

fn parse_opt_string(node: &Yaml, field: &str, i: usize, key: &str) -> Result<Option<String>> {
    match node {
        Yaml::BadValue | Yaml::Null => Ok(None),
        Yaml::String(s) => Ok(Some(s.clone())),
        _ => Err(anyhow!(".{}[{}].{} must be a string", field, i, key)),
    }
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
        assert!(matches!(spec.guest_flows[0].auth, SpecAuth::Open));
        assert_eq!(
            spec.guest_flows[0].permissions,
            BTreeSet::from([Permission::View, Permission::Download])
        );

        assert_eq!(spec.account_flows.len(), 1);
        assert!(matches!(
            &spec.account_flows[0].auth,
            SpecAuth::Password(pw) if pw == "hunter2"
        ));
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
    fn spec_root_is_the_containing_dir() {
        // the spec lives inside the dir it shares; the stem is just an arbitrary name
        assert_eq!(
            spec_root(Path::new("photos/public.share.caby")),
            Some(PathBuf::from("photos"))
        );
        // a spec at the space root shares the whole space
        assert_eq!(
            spec_root(Path::new("public.share.caby")),
            Some(PathBuf::from(""))
        );
    }

    #[test]
    fn spec_root_rejects_a_nameless_or_non_spec() {
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

    #[test]
    fn parses_a_pre_hashed_password() {
        let spec = ShareSpec::try_parse(
            "guest_flows:\n  - password_hash: $argon2id$abc\n    permissions: [view]",
        )
        .unwrap();
        assert!(matches!(
            &spec.guest_flows[0].auth,
            SpecAuth::Hash(hash) if hash == "$argon2id$abc"
        ));
    }

    #[test]
    fn rejects_both_password_and_hash() {
        assert!(ShareSpec::try_parse(
            "guest_flows:\n  - password: hunter2\n    password_hash: $argon2id$abc\n    permissions: [view]"
        )
        .is_err());
    }

    #[test]
    fn emits_yaml_that_parses_back_equivalently() {
        let spec = ShareSpec {
            account_flows: vec![SpecFlow {
                auth: SpecAuth::Hash("$argon2id$abc".to_owned()),
                permissions: BTreeSet::from([Permission::View, Permission::Write]),
                limits: Some(ShareLimits {
                    max_file_bytes: Some(1024),
                    max_bytes_per_day: None,
                    max_files_per_day: Some(5),
                }),
            }],
            guest_flows: vec![SpecFlow {
                auth: SpecAuth::Open,
                permissions: BTreeSet::from([Permission::View]),
                limits: None,
            }],
            expires_at: Some("2026-12-31T00:00:00Z".parse().unwrap()),
        };

        let yaml = String::try_from(&spec).unwrap();
        let round = ShareSpec::try_parse(&yaml).unwrap();

        assert_eq!(round.expires_at, spec.expires_at);
        assert!(matches!(
            &round.account_flows[0].auth,
            SpecAuth::Hash(hash) if hash == "$argon2id$abc"
        ));
        assert_eq!(
            round.account_flows[0].permissions,
            spec.account_flows[0].permissions
        );
        assert_eq!(round.account_flows[0].limits, spec.account_flows[0].limits);
        assert!(matches!(round.guest_flows[0].auth, SpecAuth::Open));
        assert_eq!(
            round.guest_flows[0].permissions,
            spec.guest_flows[0].permissions
        );
    }
}
