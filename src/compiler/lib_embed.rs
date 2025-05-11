#[derive(rust_embed::RustEmbed)]
#[folder = "coala_lib_std"]
struct CoalaStdLib;


pub fn get_std_lib_file(path: &str) -> Option<String> {
    let file = CoalaStdLib::get(path)?;

    return Some(String::from_utf8_lossy(&file.data).into_owned());
}