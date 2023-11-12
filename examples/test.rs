use std::io::Write;

use picopb::{
    encode::{EncodeBuffer, Result as EncoderResult, ToWire},
    wiretypes::{Field, ToVarint, WireType, WireTyped},
};

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
pub struct Response {
    pub value: [u8; 64],
    pub opaque: bytes::Bytes,
    pub error: Option<Error>,
}
pub struct RepeatedResponse {
    pub responses: [Response; 64],
}
pub struct Query {
    pub key: [u8; 8],
    pub opaque: [u8; 8],
}

impl WireTyped for &Query {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}

impl ToWire for &Query {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        buf.encode_tag(self.key.as_slice(), Field(1))?;
        let count = buf.write(&self.key)?;
        buf.encode_tag(self.opaque.as_slice(), Field(2))?;
        Ok(count + buf.write(&self.opaque)?)
    }

    fn precalculate_size(self) -> usize {
        self.key.len() + self.opaque.len()
    }
}

struct Test1 {
    a: i32,
}

impl WireTyped for &Test1 {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}

impl ToWire for &Test1 {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        buf.encode_tag(self.a, Field(1))?;
        let (a_varint, a_size) = self.a.to_varint_encoding();
        let count = buf.write(&a_varint[0..a_size])?;
        Ok(count)
    }

    fn precalculate_size(self) -> usize {
        let (_, a_size) = self.a.to_varint_encoding();
        a_size
    }
}

struct Test2 {
    b: String,
}

impl WireTyped for &Test2 {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}

impl ToWire for &Test2 {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        let count = buf.encode_tag(&self.b, Field(2))?;
        let count = count + self.b.append(buf)?;
        Ok(count)
    }

    fn precalculate_size(self) -> usize {
        let (_, b_size) = (self.b.len() as u64).to_varint_encoding();
        b_size
    }
}

struct Test3<'a> {
    b: &'a [u8],
}

impl WireTyped for &Test3<'_> {
    fn wiretype(&self) -> WireType {
        WireType::VarInt
    }
}

impl ToWire for &Test3<'_> {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        let count = buf.encode_tag(self.b, Field(2))?;
        let count = count + self.b.append(buf)?;
        Ok(count)
    }

    fn precalculate_size(self) -> usize {
        let (_, b_size) = (self.b.len() as u64).to_varint_encoding();
        b_size
    }
}

fn main() {
    let mut static_buffer: [u8; 32] = [0; 32];
    let mut buffer = EncodeBuffer::from_static(&mut static_buffer);

    // let q: Query = Query { key: [b'A'; 8], opaque: [b'B'; 8] };
    // buffer.encode(&q).unwrap();

    // let m = Test1 {a: 150};
    // buffer.encode(&m).unwrap();
    // println!("a buffer: {:#04X?}", &buffer.as_slice());

    // let mut static_buffer: [u8; 32] = [0; 32];
    // let mut buffer = EncodeBuffer::from_static(&mut static_buffer);
    // let m = Test2 {b: "testing".into()};
    // buffer.encode(&m).unwrap();
    // println!("b buffer: {:#04X?}", &buffer.as_slice());

    // let mut static_buffer: [u8; 32] = [0; 32];
    // let mut buffer = EncodeBuffer::from_static(&mut static_buffer);
    // let m = Test3 {b: "testing".as_bytes()};
    // buffer.encode(&m).unwrap();
    // println!("c buffer: {:#04X?}", &buffer.as_slice());

    // let out = std::io::stdout();
    // std::io::stdout().write_all(buffer.as_slice()).unwrap();

    // let s: String = "hello world".into();

    // buffer.encode(s).unwrap();
    // // dbg!(&static_buffer);
    // dbg!(buffer);
}
