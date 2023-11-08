use bytes::Bytes;


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

impl EncodeBuffer<'_> {
    pub fn from_static<'a>(static_buffer: &'a mut [u8]) -> EncodeBuffer<'a> {
        EncodeBuffer { 
            buffer: static_buffer,
            len: 0,
        }
    }

    /// write appends append_bytes to the encode buffer and returns the number of bytes written
    pub fn write(&mut self, append_bytes: &[u8]) -> Result<usize> {
        let count = append_bytes.len();
        let new_len = self.len + count;
        if new_len >= self.buffer.len() {
            return Err(EncodeError::BufferOutOfSpace);
        }
        self.buffer[self.len..new_len].copy_from_slice(append_bytes);
        self.len = new_len;
        Ok(count)
    }

    pub fn encode(&mut self, encodable: impl Encoder) -> Result<usize> {
        encodable.encode_append(self)
    } 
}

pub trait Encoder {
    // encodes to the end of the encode buffer and returns the number of bytes written
    fn encode_append(self, buf: &mut EncodeBuffer) -> Result<usize>;

    // precalculates the number of bytes required to encode this
    // will be called before encode_append in some cases
    fn precalculate_size(self) -> usize;
}

impl Encoder for String {
    fn encode_append(self, buf: &mut EncodeBuffer) -> Result<usize> {
        buf.write(self.as_bytes())
    }

    fn precalculate_size(self) -> usize {
        self.as_bytes().len()
    }
}
