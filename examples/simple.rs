/// A simple example of decoding a message
///
/// picopb_generate!(
/// message Login {
///     required string username = 1;
///     required string password = 1;
/// }
/// );
/// Generates:
/// struct Login {
///     username: String;
///     password: string;
/// }

/// picopb_generate!(
/// message Login {
///     required string username = 1; [(nanopb).max_size=64]
///     required string password = 1; [(nanopb).max_size=64]
/// }
/// );
/// Generates:
/// struct Login {
///     username: ArrayString;
///     password: ArrayString;
/// }

fn main() {}
