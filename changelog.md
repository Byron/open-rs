## v2.0.0 (2021-07-25)

**Breaking**: Change result from `io::Result<ExitStatus>` to `io::Result<()>`.
Commands that exit with a successful exit status result in `Ok`, otherwise an `Err` variant is created.
Previously it was easy to receive an `Ok(ExitStatus)` but forget to actually check the status. Along with
issues with particular programs reporting success even on error, doing error handling correctly was
close to impossible.
This releases alleviates most of the issues.

Error information is gathered from the stderr channel of the process using a single read operation.
This prevents the child process from keeping the main process alive if stderr is kept open.

## Notes

`wslview` always reports a 0 exit status, even if the path does not exist, which results in false positives.
The stderr channel is written to but ignored in this implementation because of the successful exit status.
Other programs write to stderr as part of normal operation. Firefox on Linux never seems to be able to load all 
the modules it wants to, so it writes to stderr but successfully opens. The only way to avoid these false positives
is to special case wsl or for `wslview` to fix the unsuccessful exit status bug.
This is only a minor problem and can mostly be ignored.

## v1.7.0 (2021-07-17)

* improved support [for
  windows-subsystem-for-linux](https://github.com/Byron/open-rs/pull/33#issue-691044025)

## v1.7.0 (2021-04-18)

* Add `gio` support on unix platforms

## v1.6.0 (2021-03-10)

* Add IOS support
* Restore Android support

## v1.5.1 (2021-03-03) - YANKED

YANKED as it would erroneously exclude Android from the list of supported platforms, making it a breaking release for some despite
the minor version change.

* Use shell instead of explorer on windows, reverting the original behaviour.

## v1.5.0 (2021-02-28) - YANKED

YANKED to avoid potential for breakage by using 'explorer.exe' to open URLs.

* Use 'explorer' on Windows instead of a shell.

## v1.4.0 (2020-03-08)

* add `open::with(path, app)` and `open::with_in_background(…)`

## v1.3.4 (2020-02-11)

* Add LICENSE.md and README.md into the crates.io tarball.

## v1.3.3 (2020-02-01)

* update code and crate to Edition 2018

## v1.2.0 (2017-01-31)

* **windows**: escape '&' in URLs. On windows, a shell is used to execute the command, which
  requires certain precautions for the URL to open to get through the interpreter.


<a name="v1.1.1"></a>
### v1.1.1 (2016-04-10)


#### Bug Fixes

* **cargo:**  no docs for open ([31605e0e](https://github.com/Byron/open-rs/commit/31605e0eddfb0cf8db635dd4d86131bc46beae78))

#### Improvements

* **api:**  allow OSStrings instead of &str ([1d13a671](https://github.com/Byron/open-rs/commit/1d13a671f2c9bd9616bf185fac77b32da1dcf8ee))



<a name="25c0e398"></a>
## 25c0e398 (2015-07-08)


#### Features

* **open**  added 'open' program ([a4c3a352](https://github.com/Byron/open-rs/commit/a4c3a352c8f912211d5ab48daaf41cb847ebcc0c))

#### Bug Fixes

* **cargo**  description added ([0fcafb56](https://github.com/Byron/open-rs/commit/0fcafb56cdb5d154b3e983d17c93a1dd7c665426))
* **open**
  *  use result ([25c0e398](https://github.com/Byron/open-rs/commit/25c0e398856c24a2daf0444640567ed3fd2f4307))
  *  don't use 'open' on linux ([30c96b1c](https://github.com/Byron/open-rs/commit/30c96b1cb95c1e03bede218b8fb03bbd9ada9317))
  *  linux uses open before anything else ([4696d1a5](https://github.com/Byron/open-rs/commit/4696d1a5ec80691e97bb1be4261d4f79ee0ade4d))
* **rustup**  (07560d233 2015-04-20) (built 2015-04-19) ([8b4e1558](https://github.com/Byron/open-rs/commit/8b4e1558f09937c555ab381ea6399a2c0758c23d))



