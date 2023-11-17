use picopb::encode::Encode;
use picopb::encode::ToWire;
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
impl From<Error> for usize {
    fn from(val: Error) -> Self {
        match val {
            Error::ErrorInvalidKey => 1,
            Error::ErrorNotFound => 2,
        }
    }
}
pub struct Query {
    pub key: [u8; 8],
    pub opaque: [u8; 8],
}
pub struct QueryFieldsType {
    pub key: picopb::common::ConstMessageField,
    pub opaque: picopb::common::ConstMessageField,
}
const QUERY_FIELDS: QueryFieldsType = QueryFieldsType {
    key: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Required,
        field_type: picopb::common::FieldType::Bytes(8),
        identifier: "key",
        ordinal: picopb::common::Field(1),
    },
    opaque: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Required,
        field_type: picopb::common::FieldType::Bytes(8),
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
        total_size += buf.encode(self.key.as_slice(), self.fields().key.ordinal)?;
        total_size += buf.encode(self.opaque.as_slice(), self.fields().opaque.ordinal)?;
        Ok(total_size)
    }
    fn precalculate_size(&self) -> usize {
        let mut total_size = 0;
        total_size += self.key.as_slice().precalculate_size();
        total_size += self.opaque.as_slice().precalculate_size();
        total_size
    }
}

use std::io::Write;
fn main() {
    let q = Query {
        key: [b'A'; 8],
        opaque: [b'B'; 8],
    };
    let mut static_buffer: [u8; 512] = [0; 512];
    let mut buffer = picopb::encode::EncodeBuffer::from_static(&mut static_buffer);
    (&q).encode(&mut buffer).expect("not error");

    eprintln!("b buffer: {:#04X?}", &buffer.as_slice());
    std::io::stdout().write_all(buffer.as_slice()).unwrap();
}
