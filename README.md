# general-sam

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-informational?style=flat-square)

A general suffix automaton implementation in Rust.

Python bindings and some utilities are also available.
Please check out [`pybind`](./pybind/).

|         [![the suffix automaton of abcbc][sam-of-abcbc]][sam-oi-wiki]          |
| :----------------------------------------------------------------------------: |
| The suffix automaton of abcbc, image from [后缀自动机 - OI Wiki][sam-oi-wiki]. |

[sam-of-abcbc]: https://oi-wiki.org/string/images/SAM/SA_suffix_links.svg
[sam-oi-wiki]: https://oi-wiki.org/string/sam/

## Examples

```rust
use general_sam::sam::GeneralSAM;

let sam = GeneralSAM::construct_from_bytes("abcbc");
// => GeneralSAM<u8>

assert!(sam.get_root_state().feed_bytes("cbc").is_accepting());
assert!(!sam.get_root_state().feed_bytes("bcb").is_accepting());
```

```rust
use general_sam::sam::GeneralSAM;

let sam = GeneralSAM::construct_from_chars("abcbc".chars());
// => GeneralSAM<char>

let state = sam.get_root_state();
let state = state.feed_chars("b");
assert!(!state.is_accepting());
let state = state.feed_chars("c");
assert!(state.is_accepting());
let state = state.feed_chars("bc");
assert!(state.is_accepting());
let state = state.feed_chars("bc");
assert!(!state.is_accepting() && state.is_nil());
```

```rust
use general_sam::{sam::GeneralSAM, trie::Trie};

let mut trie = Trie::default();
trie.insert_iter("hello".chars());
trie.insert_iter("Chielo".chars());

let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());

assert!(sam.get_root_state().feed_chars("lo").is_accepting());
assert!(sam.get_root_state().feed_chars("ello").is_accepting());
assert!(sam.get_root_state().feed_chars("elo").is_accepting());

assert!(!sam.get_root_state().feed_chars("el").is_accepting());
assert!(!sam.get_root_state().feed_chars("el").is_nil());

assert!(!sam.get_root_state().feed_chars("bye").is_accepting());
assert!(sam.get_root_state().feed_chars("bye").is_nil());
```

## References

- [Mehryar Mohri, Pedro Moreno, Eugene Weinstein.
  General suffix automaton construction algorithm and space bounds.][paper]
- 刘研绎《后缀自动机在字典树上的拓展》
- [广义后缀自动机 - OI Wiki][general-sam-oi-wiki]

[paper]: https://doi.org/10.1016/j.tcs.2009.03.034
[general-sam-oi-wiki]: https://oi-wiki.org/string/general-sam/

## License

- &copy; 2023 Chielo Newctle <ChieloNewctle@gmail.com>
- &copy; 2023 ModelTC Team

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.
