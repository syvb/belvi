// SPDX-License-Identifier: Apache-2.0
// decodes arbitrary BER
use bcder::{decode::Constructed, decode::Content, Mode};
use log::trace;

use super::{html_escape::HtmlEscapable, render_array, Render};

fn take_cons(cons: &mut Constructed<bytes::Bytes>) -> Result<String, bcder::decode::Error> {
    if let Ok(()) = cons.take_null() {
        return Ok(r#"<span class="bvcert-null">NULL</span>"#.to_string());
    }

    macro_rules! forward_to_render {
        ($($($thing:ident)::+),+,) => {
            $(
                if let Ok(thing) = $($thing ::)+take_from(cons) {
                    return Ok(thing.render());
                }
            )+
        };
    }

    forward_to_render![
        bcder::Ia5String,
        bcder::NumericString,
        bcder::PrintableString,
        bcder::Utf8String,
        bcder::OctetString,
        bcder::Oid,
        bcder::BitString,
        bcder::Integer,
        x509_certificate::asn1time::UtcTime,
    ];

    if let Ok(thing) =
        x509_certificate::asn1time::GeneralizedTime::take_from_allow_fractional_z(cons)
    {
        return Ok(thing.render());
    }

    if let Ok(s) = cons.take_sequence(|subcons| {
        let mut table = Vec::new();
        loop {
            match take_cons(subcons) {
                Ok(val) => table.push(val),
                Err(bcder::decode::Error::Malformed) => break,
                Err(bcder::decode::Error::Unimplemented) => {
                    table.push("unimplemented BER".to_string());
                    break;
                }
            }
        }
        Ok(render_array(table.into_iter()))
    }) {
        return Ok(s);
    }

    cons.take_value(|tag, content| {
        if tag.is_context_specific() {
            match content {
                Content::Primitive(prim) => {
                    let bytes = prim.take_all()?;
                    Ok(match String::from_utf8(bytes.to_vec()) {
                        Ok(str) => str.html_escape(),
                        Err(_) => bytes.render(),
                    })
                }
                Content::Constructed(_) => Err(bcder::decode::Error::Unimplemented), // TODO
            }
        } else {
            match content {
                Content::Primitive(prim) => prim.skip_all(),
                Content::Constructed(cons) => cons.skip_all(),
            }?;
            Err(bcder::decode::Error::Unimplemented)
        }
    })
}

pub fn render_ber(bytes: bytes::Bytes) -> String {
    let orig_bytes = bytes.clone();
    trace!("rendering ber {:x}", bytes);
    if let Ok(text) = Constructed::decode(bytes, Mode::Ber, take_cons) {
        text
    } else {
        format!("Unparsed DER: {}", orig_bytes.render())
    }
}

macro_rules! string_type {
    ($str:ident) => {
        impl Render for bcder::$str {
            fn render(&self) -> String {
                String::from_utf8(self.to_bytes().to_vec())
                    .unwrap()
                    .html_escape()
            }
        }
    };
}
string_type!(Ia5String);
string_type!(NumericString);
string_type!(PrintableString);
string_type!(Utf8String);
