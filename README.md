# `cmdstan-rs`: Rust interface for `CmdStan`

Suppose that you write Rust code. Suppose that you use `Stan` for
probabilistic programming. You have two choices for creating an
application which utilizes both: shell scripts, or, a Rust interface
for `CmdStan`. This crate enables the latter.

At present, this crate enables the user to compile Stan programs, run
`CmdStan` tools (diagnose, stansummary), and run any analysis method
supported by Stan. In particular, for the analysis methods, the full
flavor of arguments is provided, commensurate with builders which
supply defaults to all values the user does not wish to bother with.

See the examples for illustration.

## Future work
- serialization of data; `serde_json` is the obvious choice for
  handling JSON, but some thought is necessary in order to present in
  a coherent interface which accommodates Stan data types while being
  idiomatic in Rust.
- deserialization of Stan CSV format; this is probably within the
  scope of this crate, albeit, to be strongly typed and handle
  arbitrary Stan data types, we will likely need force the user to
  provide a type which implements `Deserialize`.

## Disclaimer
This crate is not affiliated with the Stan project.
