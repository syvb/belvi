// SPDX-License-Identifier: Apache-2.0
//! Rendering of various CT-related things.

use x509_certificate::{certificate::X509Certificate, rfc5280::Certificate};

mod arrays;
pub(crate) mod ber;
mod extensions;
pub mod html_escape;
mod oid;
mod strings;
mod time;

/// Render a key-value table.
fn render_kv_table(rows: impl Iterator<Item = (String, String)>) -> String {
    let rows_html = rows
        .map(|(k, v)| {
            format!(
                r#"<tr><th><span class="bvcert-kv-th-text">{}</span></th><td>{}</td></tr>"#,
                k, v
            )
        })
        .fold(String::new(), |a, b| a + &b);
    if rows_html.is_empty() {
        r#"<span class="bvcert-empty">(empty)</span>"#.to_string()
    } else {
        format!(r#"<table class="bvcert-kv-table">{}</table>"#, rows_html)
    }
}

fn render_array(rows: impl Iterator<Item = String>) -> String {
    render_kv_table(
        rows.enumerate()
            .map(|(idx, val)| (format!("{}.", idx), val)),
    )
}

pub trait Render {
    fn render(&self) -> String;
}

impl<T> Render for Option<T>
where
    T: Render,
{
    fn render(&self) -> String {
        match self {
            Some(val) => val.render(),
            None => r#"<span class="bvcert-empty">(none)</span>"#.to_string(),
        }
    }
}

impl Render for X509Certificate {
    fn render(&self) -> String {
        let cert: &Certificate = self.as_ref();
        cert.render()
    }
}

impl Render for Certificate {
    fn render(&self) -> String {
        render_kv_table(
            [
                (
                    "Signed certificate".to_string(),
                    self.tbs_certificate.render(),
                ),
                (
                    "Signature algorithm".to_string(),
                    self.signature_algorithm.render(),
                ),
                ("Signature".to_string(), self.signature.render()),
            ]
            .into_iter(),
        )
    }
}

impl Render for x509_certificate::rfc5280::TbsCertificate {
    fn render(&self) -> String {
        let mut table = vec![
            ("Version".to_string(), self.version.render()),
            ("Serial number".to_string(), self.serial_number.render()),
            ("Signature algorithm".to_string(), self.signature.render()),
            ("Issuer".to_string(), self.issuer.render()),
            ("Validity".to_string(), self.validity.render()),
            ("Subject".to_string(), self.subject.render()),
            (
                "Subject public key".to_string(),
                self.subject_public_key_info.render(),
            ),
        ];
        if let Some(val) = &self.issuer_unique_id {
            table.push(("Issuer ID".to_string(), val.render()));
        }
        if let Some(val) = &self.subject_unique_id {
            table.push(("Subject ID".to_string(), val.render()));
        }
        if let Some(val) = &self.extensions {
            table.push(("Extensions".to_string(), val.render()));
        }
        render_kv_table(table.into_iter())
    }
}

impl Render for x509_certificate::rfc3280::AttributeTypeAndValue {
    fn render(&self) -> String {
        render_kv_table(
            [
                ("Type".to_string(), self.typ.render()),
                ("Value".to_string(), ber::render_ber((**self.value).clone())),
            ]
            .into_iter(),
        )
    }
}

impl Render for x509_certificate::rfc5280::Validity {
    fn render(&self) -> String {
        render_kv_table(
            [
                ("Not before".to_string(), self.not_before.render()),
                ("Not after".to_string(), self.not_after.render()),
            ]
            .into_iter(),
        )
    }
}

impl Render for x509_certificate::rfc5280::Version {
    fn render(&self) -> String {
        format!("{:?}", self) // V1/V2/V3
    }
}

impl Render for x509_certificate::rfc5280::AlgorithmIdentifier {
    fn render(&self) -> String {
        let mut table = vec![("Algorithm".to_string(), self.algorithm.render())];
        if let Some(params) = &self.parameters {
            table.push((
                "Algorithm identifier".to_string(),
                ber::render_ber((***params).clone()),
            ));
        }
        render_kv_table(table.into_iter())
    }
}

impl Render for x509_certificate::rfc5280::SubjectPublicKeyInfo {
    fn render(&self) -> String {
        render_kv_table(
            [
                ("Algorithm".to_string(), self.algorithm.render()),
                (
                    "Subject public key".to_string(),
                    self.subject_public_key.render(),
                ),
            ]
            .into_iter(),
        )
    }
}
