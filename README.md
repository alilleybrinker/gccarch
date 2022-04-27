# `gccarch`

`gccarch` is a tool to query information about architectures
supported by the GNU Compiler Collection (GCC), as described
in the GCC documentation under
["Status of Supported Architectures from Maintainers' Point of View"][gcc].

`gccarch` makes it easier to query the information presented
in the ASCII-art table on that page.

## How to Use

`gccarch` supports the following flags, which are exclusive of each other.

* `-a <NAME>`/`--arch <NAME>`: print what features are supported by an 
  architecture.
* `-A`/`--archs`: print all architectures supported by GCC.
* `-f <NAME>`/`--feat <NAME>`: print all architectures which support a feature.
* `-F`/`--feats`: print all features tracked by GCC.

`gccarch` also supports the usual convenience commands:

* `-h`/`--help`: print the help text.
* `-V`/`--version`: print the current version.

## Installing

`gccarch` can be installed using a Rust toolchain. First you'll need to
[install Rust][rust_install]. Then run the following:

```bash
$ cargo install gccarch
```

`gccarch` should now be installed. You can confirm this with
`cargo install --list | grep gccarch`. Make sure your Cargo binary
directory is in your `PATH` environment variable.

## License

`gccarch` is dual-licensed with the MIT and Apache 2.0 licenses.

[gcc]: https://gcc.gnu.org/backends.html
[rust_install]: https://www.rust-lang.org/tools/install
