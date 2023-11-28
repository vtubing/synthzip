# synthzip [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/synthzip.svg
[crates.io]: https://crates.io/crates/synthzip

## What is it?

A rust library for constructing a synthetic Central Directory when you have ZIP
data that has a Local File Header (LFH), but no Central Directory File Header (CDFH),
or End of Central Directory (EOCD) data.

This is a somewhat niche library. You probably won't need it (directly), unless
you happen to be doing something unusual. But if you are, it may be **exactly**
what you need to do that one weird thing.

## How do I obtain this majestic tool?

Run the following Cargo command in your project directory (assuming you have [cargo-edit](https://github.com/killercup/cargo-edit) installed):

```fish
cargo add synthzip
```

Or add the following line to your `Cargo.toml` (in the `[dependencies]` array):

```toml
synthzip = "^ 0.1"
```

## How do I use it?

```rust
fn main() {
  // This can be any source that implements Read + Seek, and reads a valid ZIP
  // Local File Header, compressed data, and (optional) trailing Data Descriptor.
  let mut input = std::io::Cursor::new(Vec::new());

  // read the data into an entry.
  let entry = synthzip::Entry::read(&mut input).expect("failed to read zip entry from input");

  // create a new (empty) CentralDirectory
  let mut index = synthzip::CentralDirectory::new();

  // add the Entry to the CentralDirectory. This will create a corresponding ZIP
  // Central Directory File Header and update the End of Central Directory appropriately.
  index.add(&entry)

  // this can be any destination that implements Write + Seek.
  let mut output = std::fs::File::create("/path/for/output.zip").expect("failed to create output file");

  // write the entry to the output destination.
  entry.write(&mut output).expect("failed to write zip entry to output");

  // write the Central Directory after the entry data.
  index.write(&mut output).expect("failed to write central directory to output");

  /// flush the output, if you're so inclined.
  output.flush().unwrap();
}
```

## License

`synthzip` is available under the MIT License. See `LICENSE.txt` for the full text.

While the license is short, it's still written in fancy lawyer-speak. If you
prefer more down-to-earth language, consider the following:

- tl;drLegal has a simple visual summary available [here](https://www.tldrlegal.com/license/mit-license).
- FOSSA has a more in-depth overview available [here](https://fossa.com/blog/open-source-licenses-101-mit-license/).
