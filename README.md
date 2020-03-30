# Rust bindings for linux vmread library

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

* If kmod\_rw feature is used, the kernel module has to be built manually from vmread repo
* For more information go to the [vmread repo](https://github.com/Heep042/vmread)
