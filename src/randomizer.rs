
use rand::distributions::Uniform;
/// To be able to verify that PicoPB is correct we make all
/// types that PicoPB supports implement a Randomize trait,
/// which generates random values.
///
/// The trait is a constructor that creates an instance with
/// random values.
///
/// This trait can then be used to generate randomized values for
/// encoding and decoding.
///
/// SHOULD NOT BE USED OUTSIDE OF TESTING
/// Because this uses Box::leak() to make it possible to return
/// 'static references.
use rand::prelude::*;

const MAX_STRING_LEN: usize = 100;
const MAX_BYTES_LEN: usize = 100;

pub trait Randomize<T> {
    fn randomized() -> T;
}

pub fn randomized<T>() -> T
where
    T: Randomize<T>,
{
    T::randomized()
}

impl Randomize<i32> for i32 {
    fn randomized() -> i32 {
        rand::random::<i32>()
    }
}

impl Randomize<u32> for u32 {
    fn randomized() -> u32 {
        rand::random::<u32>()
    }
}

impl Randomize<i64> for i64 {
    fn randomized() -> i64 {
        rand::random::<i64>()
    }
}

impl Randomize<u64> for u64 {
    fn randomized() -> u64 {
        rand::random::<u64>()
    }
}

impl Randomize<u8> for u8 {
    fn randomized() -> u8 {
        rand::random::<u8>()
    }
}

impl Randomize<String> for String {
    fn randomized() -> String {
        let rand_len = rand::random::<usize>() % MAX_STRING_LEN;
        rand::thread_rng()
            .sample_iter(Uniform::new(char::from(32), char::from(126)))
            .take(rand_len)
            .map(char::from)
            .collect::<String>()
    }
}

// impl<T> Randomize<T> for T
// where T: Distribution<T>
//  {
//     fn randomized() -> T {
//         rand::random::<T>()
//     }
// }

// /// Leaks memory
// impl Randomize<&[u8]> for &[u8] {
//     fn randomized() -> &'static [u8] {
//         let rand_len = rand::random::<usize>().min(MAX_STRING_LEN);
//         let bytes: Vec<u8> = rand::thread_rng()
//             .sample_iter(Uniform::new(0, 255)).take(rand_len).map(u8::from).collect::<Vec<_>>();
//         Box::leak(bytes.into_boxed_slice())
//     }
// }

/// Leaks memory
impl<T> Randomize<&[T]> for &[T]
where
    T: 'static + Randomize<T>,
{
    fn randomized() -> &'static [T] {
        let rand_len = rand::random::<usize>().min(MAX_STRING_LEN);
        let values = (0..rand_len).map(|_| T::randomized()).collect::<Vec<T>>();
        Box::leak(values.into_boxed_slice())
    }
}

impl Randomize<bytes::Bytes> for bytes::Bytes {
    fn randomized() -> bytes::Bytes {
        let rand_len = rand::random::<usize>().min(MAX_STRING_LEN);
        let values = (0..rand_len)
            .map(|_| randomized::<u8>())
            .collect::<Vec<u8>>();
        bytes::Bytes::from_static(Box::leak(values.into_boxed_slice()))
    }
}

impl<T> Randomize<Vec<T>> for Vec<T>
where
    T: Randomize<T>,
{
    fn randomized() -> Vec<T> {
        let rand_len = rand::random::<usize>().min(MAX_STRING_LEN);
        (0..rand_len).map(|_| T::randomized()).collect::<Vec<_>>()
    }
}

impl<T> Randomize<Option<T>> for Option<T>
where
    T: Randomize<T>,
{
    fn randomized() -> Option<T> {
        let is_none = rand::random::<bool>();
        if is_none {
            None
        } else {
            Some(T::randomized())
        }
    }
}
