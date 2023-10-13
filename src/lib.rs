//! This crate provides an implementation of a general suffix automaton.
//!
//! # References
//!
//! - [Mehryar Mohri, Pedro Moreno, Eugene Weinstein.
//!   General suffix automaton construction algorithm and space bounds.][1]
//! - 刘研绎《后缀自动机在字典树上的拓展》
//! - [广义后缀自动机][2]
//!
//! [1]: https://doi.org/10.1016/j.tcs.2009.03.034
//! [2]: https://oi-wiki.org/string/general-sam/
//!
//! # Examples
//!
//! ```rust
//! use general_sam::{sam::GeneralSAM, trie::Trie};
//!
//! let sam_from_chars = GeneralSAM::construct_from_chars("abcbc".chars());
//! // => GeneralSAM<char>
//!
//! let state = sam_from_chars.get_root_state();
//! let state = state.feed_chars("b");
//! assert!(!state.is_accepting());
//! let state = state.feed_chars("c");
//! assert!(state.is_accepting());
//! let state = state.feed_chars("bc");
//! assert!(state.is_accepting());
//! let state = state.feed_chars("bc");
//! assert!(!state.is_accepting() && state.is_nil());
//!
//! let sam_from_bytes = GeneralSAM::construct_from_bytes("abcbc");
//! assert!(sam_from_bytes.get_root_state().feed_bytes("cbc").is_accepting());
//! assert!(!sam_from_bytes.get_root_state().feed_bytes("bcb").is_accepting());
//!
//! let mut trie = Trie::default();
//!
//! trie.insert_iter("hello".chars());
//! trie.insert_iter("Chielo".chars());
//!
//! let sam_from_trie: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());
//! assert!(sam_from_trie.get_root_state().feed_chars("lo").is_accepting());
//! assert!(sam_from_trie.get_root_state().feed_chars("ello").is_accepting());
//! assert!(sam_from_trie.get_root_state().feed_chars("elo").is_accepting());
//! assert!(!sam_from_trie.get_root_state().feed_chars("bye").is_accepting());
//! ```

pub mod sam;
pub mod trie;
pub mod trie_alike;

#[cfg(test)]
mod tests;
