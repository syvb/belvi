// SPDX-License-Identifier: Apache-2.0
use crate::res;
use axum::response::Response;
use belvi_render::html_escape::HtmlEscapable;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::trace;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

fn render_domain(s: &str) -> String {
    format!(
        r#"<div class="bvfront-domain">{}</div>"#,
        s.html_escape()
            // suggest linebreaks after dots
            .replace('.', "<wbr>.")
    )
}

fn format_date(date: DateTime<Utc>) -> String {
    date.format("%k:%M, %e %b %Y").html_escape()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    Regex,
    Subdomain,
    Recent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query: Option<String>,
    pub after: Option<String>,
    pub mode: Option<QueryMode>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertData {
    leaf_hash: Vec<u8>,
    log_id: u32,
    ts: i64,
    domain: Vec<String>,
    extra_hash: Vec<u8>,
    not_before: i64,
    not_after: i64,
}

impl CertData {
    pub fn render(&self) -> String {
        let domains = self.domain.iter().fold(String::new(), |a, b| a + b + "");
        let logged_at =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.ts / 1000, 0), Utc);
        let not_before =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.not_before, 0), Utc);
        let not_after =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.not_after, 0), Utc);
        format!(
            include_str!("tmpl/cert.html"),
            domains = domains,
            ts3339 = logged_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            ts = format_date(logged_at),
            not_before3339 = not_before.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            not_before = format_date(not_before),
            not_after3339 = not_after.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            not_after = format_date(not_after),
            json = serde_json::to_string(self).unwrap().html_escape(),
            cert_link = hex::encode(&self.leaf_hash),
        )
    }
}

pub struct SearchResults {
    pub certs: Vec<CertData>,
    pub count: Option<usize>,
    pub next: Option<String>,
}

impl Query {
    pub fn url(&self) -> String {
        let qstr = serde_urlencoded::ser::to_string(self).unwrap();
        if qstr.is_empty() {
            String::new()
        } else {
            format!("/?{}", qstr)
        }
    }

    pub fn search_sync(&self, db: &Connection, limit: u32) -> Result<SearchResults, Response> {
        let mut certs_stmt = db
            .prepare_cached(include_str!("queries/recent_certs.sql"))
            .unwrap();
        let mut certs_regex_stmt = db
            .prepare_cached(include_str!("queries/recent_certs_regex.sql"))
            .unwrap();
        let mut cert_sub_stmt = db
            .prepare_cached(include_str!("queries/recent_certs_sub.sql"))
            .unwrap();
        let mut certs_count_stmt = db.prepare_cached("SELECT COUNT(*) FROM certs").unwrap();
        let mode = self.mode.unwrap_or(QueryMode::Recent);
        let after = self.after.clone().and_then(|after| {
            let (p1, p2) = after.split_once(':')?;
            Some((p1.parse::<usize>().ok()?, p2.to_string()))
        });
        trace!("after = {:?}", after);
        let (mut certs_rows, count) = match (&self.query, mode) {
            (Some(query), QueryMode::Regex) => (certs_regex_stmt.query([query]).unwrap(), None),
            (Some(query), QueryMode::Subdomain) => (
                cert_sub_stmt
                    .query([
                        [
                            belvi_db::domrev(
                                (if let Some((_, ref dom)) = after {
                                    dom
                                } else {
                                    query
                                })
                                .to_ascii_lowercase()
                                .as_bytes(),
                            ),
                            if after.is_some() {
                                Vec::new()
                            } else {
                                vec![b'.']
                            },
                        ]
                        .concat(),
                        [
                            belvi_db::domrev(query.to_ascii_lowercase().as_bytes()),
                            vec![b'/'],
                        ]
                        .concat(),
                    ])
                    .unwrap(),
                None,
            ),
            (None, QueryMode::Recent) => (
                certs_stmt.query([]).unwrap(),
                Some(
                    certs_count_stmt
                        .query_row([], |row| row.get::<_, usize>(0))
                        .unwrap(),
                ),
            ),
            // query provided but is not needed
            (Some(_), QueryMode::Recent) => {
                let mut query = (*self).clone();
                query.query = None;
                return Err(res::redirect(&query.url()));
            }
            // no query provided
            (None, _) => return Err(res::redirect("/")),
        };

        let mut certs = Vec::new();
        let mut next = None;
        loop {
            let val = match certs_rows.next() {
                Ok(Some(val)) => val,
                Ok(None) => break,
                Err(rusqlite::Error::SqliteFailure(_, err)) => return Err(res::error(err)),
                Err(e) => panic!("unexpected error fetching certs {:#?}", e),
            };
            if let Some((min_rowid, _)) = after {
                let rowid: usize = val.get(7).unwrap();
                if min_rowid == rowid {
                    // multiple domains with same name, skip earlier ones
                    certs = Vec::new();
                }
            };
            let (domain, domain_rendered) = match val.get::<_, String>(3) {
                Ok(domain) => {
                    let rendered = render_domain(&domain);
                    (Some(domain), rendered)
                }
                Err(rusqlite::Error::InvalidColumnType(_, _, rusqlite::types::Type::Null)) => {
                    (None, "(none)".to_string())
                }
                other => panic!("unexpected domain fetching error {:?}", other),
            };
            let leaf_hash = val.get(0).unwrap();
            if let Some(true) = certs
                .last()
                .map(|last: &CertData| last.leaf_hash == leaf_hash)
            {
                // extension of last
                certs.last_mut().unwrap().domain.push(domain_rendered);
            } else {
                match certs.len().cmp(&(limit as usize)) {
                    Ordering::Less => {}
                    // stop requesting rows once we get enough
                    Ordering::Equal => {
                        if mode == QueryMode::Subdomain {
                            next = Some(format!(
                                "{}:{}",
                                val.get::<_, usize>(7).unwrap(),
                                domain.unwrap_or_else(String::new),
                            ));
                        }
                        break;
                    }
                    Ordering::Greater => unreachable!(),
                }
                certs.push(CertData {
                    leaf_hash,
                    log_id: val.get(1).unwrap(),
                    ts: val.get(2).unwrap(),
                    domain: vec![domain_rendered],
                    extra_hash: val.get(4).unwrap(),
                    not_before: val.get(5).unwrap(),
                    not_after: val.get(6).unwrap(),
                });
            }
        }
        for cert in &mut certs {
            // so when displayed they are longest to shortest
            crate::domain_sort::sort(&mut cert.domain);
        }
        Ok(SearchResults { certs, count, next })
    }
}
