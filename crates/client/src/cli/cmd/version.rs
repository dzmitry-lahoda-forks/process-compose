use core::str::FromStr;

#[derive(Debug)]
pub struct Version {
    pub version: String,
    pub rest: Vec<(String, String)>,
}

/// parses version command output
impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        // Iterate over trimmed lines without collecting first
        let mut lines = s.lines().map(|line| line.trim());
        let _title = lines.next(); // skip title line
        let mut lines: Vec<(String, String)> = lines
            .map(|line| line.split(':'))
            .take(5) // until https://github.com/F1bonacc1/process-compose/pull/410
            .map(|mut parts| {
                (
                    parts.next().unwrap().to_string(),
                    parts.fold("".to_string(), |acc, part| {
                        if acc.is_empty() {
                            part.to_string()
                        } else {
                            format!("{} {}", acc, part)
                        }
                    })
                )
            })
            .collect();
        let (version, value) = lines.remove(0);
        assert!(version == "Version");
        Ok(Self {
            version: value.trim().to_string(),
            rest: lines.into_iter().collect(),
        })
    }
}

#[cfg(test)]
#[test]
fn parse_version() {
    let output = r#"Process Compose
Version:        v1.73.0
Commit:         b7a311a
Date (UTC):     20250828114630
License:        Apache-2.0
Discord:        https://discord.gg/S4xgmRSHdC
"#;

    let version: Version = output.parse().unwrap();
    assert_eq!(version.version, "v1.73.0");
}
