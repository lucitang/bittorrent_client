use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Extension {
    #[serde(rename = "m")]
    pub inner: InnerDictionnary,
    pub metadata_size: u8,
    // pub reqq: u8,
    // pub v: ByteBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InnerDictionnary {
    pub ut_metadata: u8,
    // pub ut_pex: u8,
}

#[allow(dead_code)]
pub enum ExtensionMessageType {
    Request = 0,
    Data = 1,
    Reject = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataPayload {
    /// The corresponding ExtensionMessageType
    pub msg_type: u8,
    /// Indicates which part of the metadata this message refers to
    pub piece: u8,
}

/// The 16kiB Metadata info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetadataInfo {
    /// The corresponding ExtensionMessageType
    pub msg_type: u8,

    /// Indicates which part of the metadata this message refers to
    pub piece: u8,

    /// Metadata total size
    pub total_size: i32,
}

impl From<serde_json::Value> for MetadataInfo {
    fn from(value: Value) -> Self {
        MetadataInfo {
            msg_type: value
                .get("msg_type")
                .expect("MessageType as u8")
                .as_u64()
                .unwrap() as u8,
            piece: value.get("piece").expect("u8 piece").as_u64().unwrap() as u8,
            total_size: value
                .get("total_size")
                .expect("total_size as i32")
                .as_i64()
                .unwrap() as i32,
        }
    }
}
