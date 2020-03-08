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



