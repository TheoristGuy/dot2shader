# dot2shader-cli

CLI app for `dot2shader`.

## Usage

Set up the Rust runtime environment and enter the following command.

```bash
cargo run <input image file> [setting json file]
```

## Details of configuations

The settings will be reflected in the following order:

settings in json specified in the argument > settings in `default.json` > default settings

The format of json is defined by the serialization of [`DisplayFormat`] by [`serde`].

[`DisplayFormat`]: https://iwbtshyguy.gitlab.io/dot2shader/libdoc/dot2shader/struct.DisplayConfig.html
[`serde`]: https://crates.io/crates/serde
