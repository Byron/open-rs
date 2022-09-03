use std::io::{Error, ErrorKind};
use std::{ffi::OsStr, io};

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    web_sys::window()
        .ok_or(Error::new(ErrorKind::Other, "no window found"))?
        .open_with_url(
            (path.as_ref())
                .to_str()
                .ok_or(Error::new(ErrorKind::Other, "url not utf8"))?,
        )
        .map(|_| ())
        .map_err(|_| Error::new(ErrorKind::Other, "failed to open url in browser"))
}

pub fn with<T: AsRef<OsStr>>(_path: T, _app: impl Into<String>) -> io::Result<()> {
    unimplemented!()
}
