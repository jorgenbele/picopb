use picopb::common::*;
use picopb::encode::ToWire;
use picopb::encode::Encode;
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
