use picopb::common::Field;
use picopb::encode::EncodeBuffer;

#[test]
fn encode_person_simple() {
    let mut static_buffer: [u8; 32] = [0; 32];
    let mut buffer = EncodeBuffer::from_static(&mut static_buffer);

    /*
     * message Person {
     *   required string name = 1;
     *   required int32 id = 2;
     *   required string email = 3;
     * }
     */

    let name = String::from("John Doe");
    buffer.encode(&name, Field(1)).unwrap();

    let id: i32 = 1234;
    buffer.encode(id, Field(2)).unwrap();

    let email = String::from("jdoe@example.com");
    buffer.encode(&email, Field(3)).unwrap();

    let expected_bytes = [
        0x0a, 0x08, 0x4a, 0x6f, 0x68, 0x6e, 0x20, 0x44, 0x6f, 0x65, 0x10, 0xd2, 0x09, 0x1a, 0x10,
        0x6a, 0x64, 0x6f, 0x65, 0x40, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x63, 0x6f,
        0x6d,
    ];

    assert_eq!(&expected_bytes, buffer.as_slice(),);
}

#[test]
fn encode_person_repeated() {
    let mut static_buffer: [u8; 64] = [0; 64];
    let mut buffer = EncodeBuffer::from_static(&mut static_buffer);

    /*
     * message Person {
     *   required string name = 1;
     *   required int32 id = 2;
     *   required string email = 3;
     *
     *   repeated int32 friends = 4;;
     * }
     */

    let name = String::from("John Doe");
    buffer.encode(&name, Field(1)).unwrap();

    let id: i32 = 1234;
    buffer.encode(id, Field(2)).unwrap();

    let email = String::from("jdoe@example.com");
    buffer.encode(&email, Field(3)).unwrap();

    // write friends
    let friends: [i32; 3] = [1, 2, 3];
    // write_prefix(&mut buffer, friends.len()).unwrap();
    friends.into_iter().for_each(|friend| {
        buffer.encode(friend, Field(4)).unwrap();
    });

    let expected_bytes = [
        0x0a, 0x08, 0x4a, 0x6f, 0x68, 0x6e, 0x20, 0x44, 0x6f, 0x65, 0x10, 0xd2, 0x09, 0x1a, 0x10,
        0x6a, 0x64, 0x6f, 0x65, 0x40, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x63, 0x6f,
        0x6d, 0x20, 0x01, 0x20, 0x02, 0x20, 0x03,
    ];

    assert_eq!(&expected_bytes, buffer.as_slice(),);
}

// #[test]
// fn encode_person_repeated() {
//     let mut static_buffer: [u8; 32] = [0; 32];
//     let mut buffer = EncodeBuffer::from_static(&mut static_buffer);
//
//     /*
//      * enum PhoneType {
//      *   PHONE_TYPE_UNSPECIFIED = 0,
//      *   PHONE_TYPE_MOBILE = 1,
//      *   PHONE_TYPE_HOME = 2,
//      *   PHONE_TYPE_WORK = 3,
//      *   ...
//      * }
//      * message PhoneNumber {
//      *   required string number = 1;
//      *   required PhoneType type = 2;
//      * }
//      * message Person {
//      *   required string name = 1;
//      *   required int32 id = 2;
//      *   required string email = 3;
//      *
//      *   repeated PhoneNumber numbers = 4;
//      * }
//      */
//
//     let name = String::from("John Doe");
//     buffer.encode(&name, Field(1)).unwrap();
//
//     let id: i32 = 1234;
//     buffer.encode(id, Field(2)).unwrap();
//
//     let email = String::from("jdoe@example.com");
//     buffer.encode(&email, Field(3)).unwrap();
//
//
//     write_prefix(buffer, )
//     buffer.encode_
//
//     let expected_bytes = [
//         0x0a, 0x08, 0x4a, 0x6f, 0x68, 0x6e, 0x20, 0x44, 0x6f, 0x65, 0x10, 0xd2, 0x09, 0x1a, 0x10,
//         0x6a, 0x64, 0x6f, 0x65, 0x40, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x63, 0x6f,
//         0x6d,
//     ];
//
//     assert_eq!(&expected_bytes, buffer.as_slice(),);
// }
