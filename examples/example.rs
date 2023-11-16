use picopb::common::*;
use picopb::encode::ToWire;
use picopb::encode::Encode;
use std::ops::Deref;
pub enum Error {
    ErrorInvalidKey,
    ErrorNotFound,
}
impl TryFrom<usize> for Error {
    type Error = String;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Error::ErrorInvalidKey),
            2 => Ok(Error::ErrorNotFound),
            _ => Err(format!("invalid ordinal value: {} for enum Error", value)),
        }
    }
}
impl Into<usize> for Error {
    fn into(self) -> usize {
        match self {
            Error::ErrorInvalidKey => 1,
            Error::ErrorNotFound => 2,
        }
    }
}
pub struct Query {
    pub key: bytes::Bytes,
    pub opaque: bytes::Bytes,
}
pub struct QueryFieldsType {
    pub key: picopb::common::ConstMessageField,
    pub opaque: picopb::common::ConstMessageField,
}
const QUERY_FIELDS: QueryFieldsType = QueryFieldsType {
    key: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Required,
        field_type: picopb::common::FieldType::UnboundedBytes,
        identifier: "key",
        ordinal: picopb::common::Field(1),
    },
    opaque: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Required,
        field_type: picopb::common::FieldType::UnboundedBytes,
        identifier: "opaque",
        ordinal: picopb::common::Field(2),
    },
};
impl Query {
    fn fields(&self) -> QueryFieldsType {
        QUERY_FIELDS
    }
}
impl picopb::encode::Encode for &Query {
    fn encode(&self, buf: &mut picopb::encode::EncodeBuffer) -> std::io::Result<usize> {
        let mut total_size = 0;
        total_size += buf.encode(self.key.deref(), self.fields().key.ordinal)?;
        total_size += buf.encode(self.opaque.deref(), self.fields().opaque.ordinal)?;
        Ok(total_size)
    }
    fn precalculate_size(&self) -> usize {
        let mut total_size = 0;
        total_size += self.key.deref().precalculate_size();
        total_size += self.opaque.deref().precalculate_size();
        total_size
    }
}

use std::io::Write;
use std::io;
use bytes::Bytes;
fn main() {
    let key = "key_str".as_bytes();
    let opaque = "value_str".as_bytes();

    let q = Query { key: Bytes::from_static(key), opaque: Bytes::from_static(opaque) };
    let mut static_buffer: [u8; 512] = [0; 512];
    let mut buffer = picopb::encode::EncodeBuffer::from_static(&mut static_buffer);
    (&q).encode(&mut buffer).expect("not error");

    eprintln!("b buffer: {:#04X?}", &buffer.as_slice());
    std::io::stdout().write_all(buffer.as_slice()).unwrap();
}