use picopb::common::*;
use picopb::encode::Encode;
use picopb::encode::ToWire;
use picopb::randomizer::{randomized, Randomize};
use std::ops::Deref;
#[derive(Default, Debug)]
pub enum Error {
    #[default]
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
#[derive(Default, Debug)]
pub struct Query {
    pub key: bytes::Bytes,
    pub opaque: bytes::Bytes,
}
#[derive(Debug)]
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
impl Randomize<Query> for Query {
    fn randomized() -> Query {
        Self {
            key: randomized::<bytes::Bytes>(),
            opaque: randomized::<bytes::Bytes>(),
        }
    }
}

use std::io::Write;

fn main() {
    (0..1000).for_each(|_| {
        let q = Query::randomized();
        let mut static_buffer: [u8; 20 * 10000] = [0; 20 * 10000];
        let mut buffer = picopb::encode::EncodeBuffer::from_static(&mut static_buffer);
        (&q).encode(&mut buffer).expect("not error");
        std::io::stdout().write_all(buffer.as_slice()).unwrap();
    });
}
