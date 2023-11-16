use picopb::common::Field;
use picopb::encode::EncodeBuffer;

#[test]
fn encode_string_testing() {
    let mut static_buffer: [u8; 32] = [0; 32];
    let mut buffer = EncodeBuffer::from_static(&mut static_buffer);

    /*
     * message Test2 {
     *   optional string b = 2;
     * }
     */
    let field = Field(2);
    let s = String::from("testing");
    buffer.encode(&s, field).unwrap();

    assert_eq!(
        &[0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67],
        buffer.as_slice(),
    );
}

#[test]
fn encode_string2_testing() {
    let mut static_buffer: [u8; 32] = [0; 32];
    let mut buffer = EncodeBuffer::from_static(&mut static_buffer);

    let field = Field(2);
    let s = String::from("testing2");
    buffer.encode(&s, field).unwrap();

    assert_eq!(
        &[0x12, 0x08, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, b'2'],
        buffer.as_slice(),
    );
}
