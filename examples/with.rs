extern crate open;


fn main() {

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    open::with("Cargo.toml", "kate").unwrap();

    #[cfg(target_os = "windows")]
    open::with("Cargo.toml", "notepad").unwrap();

    #[cfg(target_os = "macos")]
    open::with("Cargo.toml", "TextEdit").unwrap();

}