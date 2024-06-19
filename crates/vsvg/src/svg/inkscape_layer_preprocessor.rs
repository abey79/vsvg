use quick_xml::events::attributes::{AttrError, Attribute};
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};

static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum InkscapeExtPreprocessorError {
    #[error("XML error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttrError(#[from] AttrError),

    #[error("JSON encode error: {0}")]
    JSONEncodeError(#[from] serde_json::Error),

    #[error("UTF8 decode error: {0}")]
    UTF8DecodeError(#[from] std::str::Utf8Error),
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq)]
pub(crate) struct GroupInfo {
    pub(crate) id: Option<String>,
    pub(crate) groupmode: Option<String>,
    pub(crate) label: Option<String>,
}

impl GroupInfo {
    fn id(&mut self, id: &[u8]) {
        self.id = Some(String::from_utf8_lossy(id).to_string());
    }

    fn groupmode(&mut self, groupmode: &[u8]) {
        self.groupmode = Some(String::from_utf8_lossy(groupmode).to_string());
    }

    fn label(&mut self, label: &[u8]) {
        self.label = Some(String::from_utf8_lossy(label).to_string());
    }

    /// This function encodes a [`GroupInfo`] into a string that can be used as an `id` attribute.
    ///
    /// Note that encoding is performed even if only the `id` attribute is set, even to `""`. This
    /// is to distinguish actual, pre-existing groups against `usvg`-added groups.
    pub(crate) fn encode(self) -> Result<String, InkscapeExtPreprocessorError> {
        let encoded = if self.groupmode.is_some() || self.label.is_some() || self.id.is_some() {
            self.encode_impl(&base64::prelude::BASE64_URL_SAFE_NO_PAD)?
        } else {
            format!(
                "__vsvg_missing__{}",
                UNIQUE_ID.fetch_add(1, Ordering::Relaxed)
            )
        };

        Ok(encoded)
    }

    fn encode_impl(
        self,
        engine: &impl base64::Engine,
    ) -> Result<String, InkscapeExtPreprocessorError> {
        let json = serde_json::to_string(&self)?;
        let mut encoded = String::from("__vsvg_encoded__");
        engine.encode_string(&json, &mut encoded);

        Ok(encoded)
    }

    /// This function decodes a [`GroupInfo`] from a string that was previously encoded by
    /// [`GroupInfo::encode`]. It implements the semantics that an empty `id` attribute corresponds
    /// to a spurious, `usvg`-added group, as originally empty
    pub(crate) fn decode(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        let group_info = Self::decode_impl(s, &base64::prelude::BASE64_URL_SAFE_NO_PAD);

        Some(group_info.unwrap_or_else(|| {
            let id = if s.starts_with("__vsvg_missing__") {
                None
            } else {
                Some(s.to_owned())
            };

            GroupInfo {
                id,
                groupmode: None,
                label: None,
            }
        }))
    }

    fn decode_impl(s: &str, engine: &impl base64::Engine) -> Option<Self> {
        let s = s.strip_prefix("__vsvg_encoded__")?;
        let json = engine.decode(s.as_bytes()).ok()?;
        let group_info = serde_json::from_slice(&json).ok()?;

        Some(group_info)
    }
}

pub(crate) fn preprocess_inkscape_layer(xml: &str) -> Result<String, InkscapeExtPreprocessorError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            // handle groups
            Ok(Event::Start(e)) if e.name().as_ref() == b"g" => {
                let mut elem = BytesStart::new("g");
                let mut group_info = GroupInfo::default();
                for attr in e.attributes() {
                    match attr {
                        Ok(Attribute {
                            key: QName(b"id"),
                            value,
                        }) => group_info.id(&value),
                        Ok(Attribute {
                            key: QName(b"inkscape:groupmode"),
                            value,
                        }) => group_info.groupmode(&value),

                        Ok(Attribute {
                            key: QName(b"inkscape:label"),
                            value,
                        }) => group_info.label(&value),
                        Ok(attr) => elem.push_attribute(attr),
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }

                elem.push_attribute((b"id".as_slice(), group_info.encode()?.as_bytes()));

                writer.write_event(Event::Start(elem))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e)?,
            Err(e) => return Err(e.into()),
        }
    }

    let result = writer.into_inner().into_inner();
    let result_string = std::str::from_utf8(&result)?.to_owned();

    Ok(result_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_info_id() {
        let mut gi = GroupInfo::default();
        gi.id("test".as_bytes());
        let encoded = gi.clone().encode().unwrap();
        assert!(encoded.starts_with("__vsvg_encoded__"));

        let decoded = GroupInfo::decode(encoded.as_str());
        assert_eq!(decoded, Some(gi));
    }

    #[test]
    fn test_group_info_id_empty_str() {
        let mut gi = GroupInfo::default();
        gi.id("".as_bytes());
        let encoded = gi.clone().encode().unwrap();
        assert!(encoded.starts_with("__vsvg_encoded__"));

        let decoded = GroupInfo::decode(encoded.as_str());
        assert_eq!(decoded, Some(gi));
    }

    #[test]
    fn test_group_info_group_mode() {
        let mut gi = GroupInfo::default();
        gi.groupmode("layer".as_bytes());
        gi.label("my label".as_bytes());
        let encoded = gi.clone().encode().unwrap();
        assert!(encoded.starts_with("__vsvg_encoded__"));

        let decoded = GroupInfo::decode(encoded.as_str());
        assert_eq!(decoded, Some(gi));
    }

    #[test]
    fn test_group_info_empty() {
        let gi = GroupInfo::default();
        let encoded = gi.clone().encode().unwrap();
        assert!(encoded.starts_with("__vsvg_missing__"));

        let decoded = GroupInfo::decode(encoded.as_str());
        assert_eq!(decoded, Some(gi));
    }

    #[test]
    fn test_group_info_empty_unique() {
        let gi = GroupInfo::default();
        let encoded = gi.clone().encode().unwrap();

        let gi2 = GroupInfo::default();
        let encoded2 = gi2.clone().encode().unwrap();

        assert_ne!(encoded, encoded2);
    }

    #[test]
    fn test_group_info_decode_from_empty() {
        let gi = GroupInfo::decode("");
        assert_eq!(gi, None);
    }

    #[test]
    fn test_preprocess_inkscape_ext() {
        UNIQUE_ID.store(0, Ordering::SeqCst);
        let xml = r#"
        <?xml version="1.0" encoding="utf-8" ?>
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">
            <g></g>
            <g id=""></g>
            <g inkscape:label="3"></g>
            <g id="hello" inkscape:label="3" inkscape:groupmode="layer" stroke="white"></g>
        </svg>
        "#;

        let mut preprocessed = preprocess_inkscape_layer(xml).unwrap();

        let expected = r#"
<?xml version="1.0" encoding="utf-8" ?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">
<g id="__vsvg_missing__0"></g>
<g id="__vsvg_encoded__eyJpZCI6IiIsImdyb3VwbW9kZSI6bnVsbCwibGFiZWwiOm51bGx9"></g>
<g id="__vsvg_encoded__eyJpZCI6bnVsbCwiZ3JvdXBtb2RlIjpudWxsLCJsYWJlbCI6IjMifQ"></g>
<g stroke="white" id="__vsvg_encoded__eyJpZCI6ImhlbGxvIiwiZ3JvdXBtb2RlIjoibGF5ZXIiLCJsYWJlbCI6IjMifQ"></g>
</svg>"#.to_string().replace("\n", "");

        // force unique ID to 0 to account for other tests bumping UNIQUE_ID
        let idx = preprocessed.find("__vsvg_missing__").unwrap();
        preprocessed.replace_range(idx..idx + 17, "__vsvg_missing__0");

        assert_eq!(preprocessed, expected);
    }
}
