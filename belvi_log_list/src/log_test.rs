// SPDX-License-Identifier: Apache-2.0
use super::*;
use chrono::TimeZone;

fn validities(log: &Log) -> [bool; 13] {
    fn jan1(year: i32) -> DateTime<Utc> {
        chrono::Utc.ymd(year, 01, 01).and_hms(00, 00, 00)
    }
    [
        log.has_active_certs(jan1(2015)),
        log.has_active_certs(jan1(2016)),
        log.has_active_certs(jan1(2017)),
        log.has_active_certs(jan1(2018)),
        log.has_active_certs(jan1(2019)),
        log.has_active_certs(jan1(2020)),
        log.has_active_certs(jan1(2021)),
        log.has_active_certs(jan1(2022)),
        log.has_active_certs(jan1(2023)),
        log.has_active_certs(jan1(2024)),
        log.has_active_certs(jan1(2025)),
        log.has_active_certs(jan1(2026)),
        log.has_active_certs(jan1(2027)),
    ]
}

#[test]
fn argon2021() {
    let data = r#"
        {
            "description": "Google 'Argon2021' log",
            "log_id": "9lyUL9F3MCIUVBgIMJRWjuNNExkzv98MLyALzE7xZOM=",
            "key": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAETeBmZOrzZKo4xYktx9gI2chEce3cw/tbr5xkoQlmhB18aKfsxD+MnILgGNl0FOm0eYGilFVi85wLRIOhK8lxKw==",
            "url": "https://ct.googleapis.com/logs/argon2021/",
            "mmd": 86400,
            "state": {
                "usable": {
                    "timestamp": "2018-06-15T02:30:13Z"
                }
            },
            "temporal_interval": {
                "start_inclusive": "2021-01-01T00:00:00Z",
                "end_exclusive": "2022-01-01T00:00:00Z"
            }
        }
    "#;
    let log = serde_json::from_str::<Log>(data).unwrap();
    assert_eq!(log, Log {
        description: "Google 'Argon2021' log".to_string(),
        log_id: "9lyUL9F3MCIUVBgIMJRWjuNNExkzv98MLyALzE7xZOM=".to_string(),
        key: "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAETeBmZOrzZKo4xYktx9gI2chEce3cw/tbr5xkoQlmhB18aKfsxD+MnILgGNl0FOm0eYGilFVi85wLRIOhK8lxKw==".to_string(),
        url: "https://ct.googleapis.com/logs/argon2021/".to_string(),
        mmd: 86400,
        state: LogState::Usable {
            timestamp: "2018-06-15T02:30:13Z".to_string(),
        },
        temporal_interval: Some(TemporalInterval {
            start_inclusive: "2021-01-01T00:00:00Z".to_string(),
            end_exclusive: "2022-01-01T00:00:00Z".to_string(),
        })
    });
    assert_eq!(
        log.get_sth_url(),
        "https://ct.googleapis.com/logs/argon2021/ct/v1/get-sth".to_string()
    );
    assert_eq!(
        log.get_entries_url(1337, 31337),
        "https://ct.googleapis.com/logs/argon2021/ct/v1/get-entries?start=1337&end=31337"
            .to_string()
    );
    assert_eq!(
        validities(&log),
        [true, true, true, true, true, true, true, false, false, false, false, false, false],
    );
}

#[test]
fn aviator() {
    let data = r#"
        {
            "description": "Google 'Aviator' log",
            "log_id": "aPaY+B9kgr46jO65KB1M/HFRXWeT1ETRCmesu09P+8Q=",
            "key": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE1/TMabLkDpCjiupacAlP7xNi0I1JYP8bQFAHDG1xhtolSY1l4QgNRzRrvSe8liE+NPWHdjGxfx3JhTsN9x8/6Q==",
            "url": "https://ct.googleapis.com/aviator/",
            "mmd": 86400,
            "state": {
                "readonly": {
                    "timestamp": "2016-11-30T13:24:18Z",
                    "final_tree_head": {
                        "sha256_root_hash": "LcGcZRsm+LGYmrlyC5LXhV1T6OD8iH5dNlb0sEJl9bA=",
                        "tree_size": 46466472
                    }
                }
            }
        }
    "#;
    let log = serde_json::from_str::<Log>(data).unwrap();
    assert_eq!(log, Log {
        description: "Google 'Aviator' log".to_string(),
        log_id: "aPaY+B9kgr46jO65KB1M/HFRXWeT1ETRCmesu09P+8Q=".to_string(),
        key: "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE1/TMabLkDpCjiupacAlP7xNi0I1JYP8bQFAHDG1xhtolSY1l4QgNRzRrvSe8liE+NPWHdjGxfx3JhTsN9x8/6Q==".to_string(),
        url: "https://ct.googleapis.com/aviator/".to_string(),
        mmd: 86400,
        state: LogState::ReadOnly {
            timestamp: "2016-11-30T13:24:18Z".to_string(),
            final_tree_head: TreeHead {
                sha256_root_hash: "LcGcZRsm+LGYmrlyC5LXhV1T6OD8iH5dNlb0sEJl9bA=".to_string(),
                tree_size: 46466472,
            }
        },
        temporal_interval: None,
    });
    assert_eq!(
        validities(&log),
        [true, true, true, true, true, false, false, false, false, false, false, false, false],
    );
}

#[test]
fn nimbus2022() {
    let data = r#"
        {
            "description": "Cloudflare 'Nimbus2022' Log",
            "log_id": "QcjKsd8iRkoQxqE6CUKHXk4xixsD6+tLx2jwkGKWBvY=",
            "key": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAESLJHTlAycmJKDQxIv60pZG8g33lSYxYpCi5gteI6HLevWbFVCdtZx+m9b+0LrwWWl/87mkNN6xE0M4rnrIPA/w==",
            "url": "https://ct.cloudflare.com/logs/nimbus2022/",
            "mmd": 86400,
            "state": {
                "usable": {
                    "timestamp": "2019-10-31T19:22:00Z"
                }
            },
            "temporal_interval": {
                "start_inclusive": "2022-01-01T00:00:00Z",
                "end_exclusive": "2023-01-01T00:00:00Z"
            }
        }
    "#;
    let log = serde_json::from_str::<Log>(data).unwrap();
    assert_eq!(
        validities(&log),
        [true, true, true, true, true, true, true, true, false, false, false, false, false],
    );
}
