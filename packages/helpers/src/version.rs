use cosmwasm_std::{StdError, StdResult};

pub fn version_to_u32(version: &str) -> StdResult<u32> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return StdResult::Err(StdError::parse_err(
            version,
            "Received invalid version string",
        ));
    }

    let mut numeric_version: u32 = 0;

    for (i, part) in parts.iter().rev().enumerate() {
        let num: u32 = part.parse().map_err(|_| {
            StdError::parse_err(version, &format!("Invalid number in version: {}", part))
        })?;
        if num >= 1024 {
            return StdResult::Err(StdError::parse_err(
                version,
                "Version string invalid, number too large",
            ));
        }
        numeric_version |= num << (i * 10);
    }
    Ok(numeric_version)
}

pub fn u32_to_version_string(v: u32) -> String {
    format!("{}.{}.{}", (v >> 20) & 1023, (v >> 10) & 1023, v & 1023)
}

#[test]
fn test_version_to_u32() {
    assert_eq!(version_to_u32("1.2.3").unwrap(), 1050627);
    assert_eq!(version_to_u32("0.0.1").unwrap(), 1);
    assert!(version_to_u32("0.0.1").unwrap() < version_to_u32("0.0.2").unwrap());
    assert!(version_to_u32("0.1.0").unwrap() > version_to_u32("0.0.200").unwrap());
    assert!(version_to_u32("1.1.0").unwrap() > version_to_u32("0.200.200").unwrap());
    assert_eq!(version_to_u32("0.0.0").unwrap(), 0);
    assert!(version_to_u32("0.0").is_err());
    assert_eq!(
        u32_to_version_string(version_to_u32("1.200.120").unwrap()),
        "1.200.120"
    );
}
