# jtex
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![# Issues](https://img.shields.io/github/issues/zedseven/jtex.svg?logo=github)](https://github.com/zedseven/jtex/issues)

A crate for decoding Nintendo Jupiter Texture files.

Based on the work of [jtex_view](https://github.com/zedseven/jtex_view) and
[Kuriimu2](https://github.com/FanTranslatorsInternational/Kuriimu2/blob/dev/plugins/Nintendo/plugin_nintendo/Images/RawJtex.cs).

## Usage
```rust
let file = File::open("Prs_P_254_C.jtex")?;
let decoder = JupiterDecoder::decode(file)?;
let image = DynamicImage::from_decoder(decoder)?;

image.save_with_format("Prs_P_254_C.png", ImageFormat::Png)?;
```

Please refer to the [API Documentation](https://zedseven.github.io/jtex/) for more information.

## Project License
This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in *jtex* by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
