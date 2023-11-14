use picopb::common::*;
use picopb::encode::ToWire;
use picopb::encode::Encode;
pub struct MessageWithOptionalField {
    pub d: Option<String>,
    pub e: Vec<i32>,
}
pub struct MessageWithOptionalFieldFieldsType {
    pub d: picopb::common::ConstMessageField,
    pub e: picopb::common::ConstMessageField,
}
const MESSAGE_WITH_OPTIONAL_FIELD_FIELDS: MessageWithOptionalFieldFieldsType = MessageWithOptionalFieldFieldsType {
    d: picopb::common::ConstMessageField {
        qualifier: picopb::common::FieldQualifier::Optional,
        field_type: picopb::common::FieldType::UnboundedString,
        identifier: "d",
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
        if let Some(value_d) = &self.d {
            total_size += buf.encode(value_d.as_str(), self.fields().d.ordinal)?;
        }
        total_size += buf.encode(self.e.as_slice(), self.fields().e.ordinal)?;
        Ok(total_size)
    }
    fn precalculate_size(&self) -> usize {
        let mut total_size = 0;
        if let Some(value_d) = &self.d {
            total_size += value_d.as_str().precalculate_size();
        }
        total_size += self.e.as_slice().precalculate_size();
        total_size
    }
}
