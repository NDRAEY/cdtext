# cdtext

A rough implementation of CD-Text parser. This crate can be used to read CD-Text binary data given by CD drive.

See `examples/parse.rs` for more info.

# Getting the data

To dump the CD-Text info from your CD in Linux, open your terminal and run following command:

```bash
cdrecord dev=/dev/srX -vv -toc
```

> Where `X` is your drive number.

cdrecord will print some info into console, and create a `cdtext.dat` file.

# Parsing and working with data

Firstly, load data from somewhere by using `std::fs::read` or use any function that can give you a slice of `u8`.

Then create a parser:

```rust
let cdtext = CDText::from_data_with_length(&data);
```

Then parse:

```rust
let data: Vec<cdtext::CDTextEntry> = cdtext.parse();
```

Now data is ready for further processing.

See docs for more information.