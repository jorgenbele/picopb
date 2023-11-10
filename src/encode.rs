use std::io::Write;

use crate::wiretypes::{Field, ToVarint, WireTyped};

use leb128;

#[derive(Debug)]
pub enum EncodeError {
    BufferOutOfSpace,
}

pub type Result<T> = std::result::Result<T, EncodeError>;

#[derive(Debug)]
pub struct EncodeBuffer<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl Write for EncodeBuffer<'_> {
    fn write(&mut self, append_bytes: &[u8]) -> std::io::Result<usize> {
        let count = append_bytes.len();
        let new_len = self.len + count;
        if new_len >= self.buffer.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "out of memory",
            ));
        }
        self.buffer[self.len..new_len].copy_from_slice(append_bytes);
        self.len = new_len;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl EncodeBuffer<'_> {
    pub fn from_static<'a>(static_buffer: &'a mut [u8]) -> EncodeBuffer<'a> {
        EncodeBuffer {
            buffer: static_buffer,
            len: 0,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    pub fn encode_tag(&mut self, encodable: impl ToWire, field: Field) -> std::io::Result<usize> {
        encodable.write_tag(self, field)
    }

    pub fn encode_value(&mut self, encodable: impl ToWire) -> std::io::Result<usize> {
        encodable.append(self)
    }

    pub fn encode(
        &mut self,
        encodable: impl ToWire + Copy,
        field: Field,
    ) -> std::io::Result<usize> {
        self.encode_tag(encodable, field)?;
        self.encode_value(encodable)
    }
}

/// The ToWire trait encodes the type in the protocol buffers wire format
pub trait ToWire: WireTyped {
    /// encodes the tag for the type and returns the bytes written
    fn write_tag(&self, buf: &mut EncodeBuffer, field: Field) -> std::io::Result<usize> {
        let (bytes, count) = self.tag(field).encode();
        buf.write(&bytes[0..count])
    }

    /// encodes to the end of the encode buffer and returns the number of bytes written
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize>;

    /// precalculates the number of bytes required to encode this
    /// will be called before encode_append in some cases
    fn precalculate_size(self) -> usize;
}

/// Writes the length prefix. Used for length encoded types.
pub fn write_prefix(buf: &mut EncodeBuffer, len: usize) -> std::io::Result<usize> {
    let (bytes, count) = (len as u64).to_varint_encoding();
    buf.write(&bytes[..count])
}

impl ToWire for &String {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        write_prefix(buf, self.len())?;
        buf.write(self.as_bytes())
    }

    fn precalculate_size(self) -> usize {
        self.as_bytes().len()
    }
}

impl ToWire for &[u8] {
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        write_prefix(buf, self.len())?;
        buf.write(self)
    }

    fn precalculate_size(self) -> usize {
        self.len()
    }
}

impl ToWire for i32
where
    i32: ToVarint,
{
    fn append(self, buf: &mut EncodeBuffer) -> std::io::Result<usize> {
        leb128::write::signed(buf, self as i64)
    }

    fn precalculate_size(self) -> usize {
        let (_, size) = self.to_varint_encoding();
        size
    }
}
