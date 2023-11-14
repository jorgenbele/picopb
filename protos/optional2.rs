use picopb::common::*;
use picopb::encode::ToWire;
use picopb::encode::Encode;
pub struct MessageWithOptionalField {
    pub a: Option<String>,
    pub b: Vec<String>,
    pub e: Vec<i32>,
}
pub struct MessageWithOptionalFieldFieldsType {
    pub a: picopb::common::ConstMessageField,
    pub b: picopb::common::ConstMessageField,
    pub e: picopb::common::ConstMessageField,
}
const MESSAGE_WITH_OPTIONAL_FIELD_FIELDS: MessageWithOptionalFieldFieldsType = MessageWithOptionalFieldFieldsType {
    a: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Optional,
        field_type: picopb::common::FieldType::UnboundedString,
        identifier: "a",
        ordinal: picopb::common::Field(1),
    },
    b: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::RepeatedUnbounded,
        field_type: picopb::common::FieldType::UnboundedString,
        identifier: "b",
        ordinal: picopb::common::Field(4),
    },
    e: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::RepeatedUnbounded,
        field_type: picopb::common::FieldType::Int32,
        identifier: "e",
        ordinal: picopb::common::Field(5),
    },
};
impl MessageWithOptionalField {
    fn fields(&self) -> MessageWithOptionalFieldFieldsType {
        MESSAGE_WITH_OPTIONAL_FIELD_FIELDS
    }
}
impl picopb::encode::Encode for &MessageWithOptionalField {
    fn encode(&self, buf: &mut picopb::encode::EncodeBuffer) -> std::io::Result<usize> {
        let mut total_size = 0;
        if let Some(value_a) = &self.a {
            total_size += buf.encode(value_a.as_str(), self.fields().a.ordinal)?;
        }
        total_size += buf.encode(self.b.as_slice(), self.fields().b.ordinal)?;
        total_size += buf.encode(self.e.as_slice(), self.fields().e.ordinal)?;
        Ok(total_size)
    }
    fn precalculate_size(&self) -> usize {
        let mut total_size = 0;
        if let Some(value_a) = &self.a {
            total_size += value_a.as_str().precalculate_size();
        }
        total_size += self.b.as_slice().precalculate_size();
        total_size += self.e.as_slice().precalculate_size();
        total_size
    }
}
