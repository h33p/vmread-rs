# Rust bindings for linux vmread library


[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/vmread.svg
[crates-url]: https://crates.io/crates/vmread
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE.md

[API Docs](https://docs.rs/vmread/latest/vmread)

## Main crates

* vmread: Safe high-level API
* vmread-sys: Unsafe generated low-level API

## Examples

Build the examples with the following command:

```
cargo build --examples <--features kmod_rw>
```

Be sure to run them as root, they will be placed in target/(debug|release)/examples/ directory

## More information

* If kmod\_rw feature is used, the required kernel module gets built inside target vmread-sys directory
* For more information go to the [vmread repo](https://github.com/Heep042/vmread)
