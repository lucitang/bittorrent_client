use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Extension {
    #[serde(rename = "m")]
    pub inner: InnerDictionnary,
    // pub metadata_size: u8,
    // pub reqq: u8,
    // pub v: ByteBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InnerDictionnary {
    pub ut_metadata: u8,
    // pub ut_pex: u8,
}
