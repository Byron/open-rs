[![Build Status](https://travis-ci.org/Byron/open-rs.svg?branch=master)](https://travis-ci.org/Byron/open-rs)

Use this library to open a path or URL using the program configured on the system. It is equivalent to running one of the following:

```bash
# OSX
$ open <path-or-url>
# Windows
$ start <path-or-url>
# Linux
$ open <path-or-url> || xdg-open <path-or-url> || gnome-open <path-or-url> || kde-open <path-or-url>
```

# Usage

Add this to your Cargo.toml
```toml
[dependencies]
open = "*"
```

Add this to your lib ...
```Rust
extern crate open;
```

... and open something using
```Rust
open::that("https://rust-lang.org");
```

Follow this link for the [massive API docs](http://byron.github.io/open-rs).

# Credits

The implementation is based on the respective functionality of [cargo](https://github.com/rust-lang/cargo), but was improved to allow some error handling.