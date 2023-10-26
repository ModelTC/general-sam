//! This crate provides an implementation of a general suffix automaton.
//!
//! ```mermaid
//! flowchart LR
//!   init((ε))
//!   a((a))
//!   b((b))
//!   ab((ab))
//!   bc(((bc)))
//!   abc((abc))
//!   abcb((abcb))
//!   abcbc(((abcbc)))
//!
//!   init -- a --> a
//!   init -- b --> b
//!   a -- b --> ab
//!   b -- c --> bc
//!   init -- c --> bc
//!   ab -- c --> abc
//!   bc -- b --> abcb
//!   abc -- b --> abcb
//!   abcb -- c --> abcbc
//! ```
//!
//! > The suffix automaton of abcbc.
//!
//! # Examples
//!
//! ```rust
//! use general_sam::sam::GeneralSAM;
//!
//! let sam = GeneralSAM::construct_from_bytes("abcbc");
//! // => GeneralSAM<u8>
//!
//! // "cbc" is a suffix of "abcbc"
//! assert!(sam.get_root_state().feed_bytes("cbc").is_accepting());
//!
//! // "bcb" is not a suffix of "abcbc"
//! assert!(!sam.get_root_state().feed_bytes("bcb").is_accepting());
//! ```
//!
//! ```rust
//! use general_sam::sam::GeneralSAM;
//!
//! let sam = GeneralSAM::construct_from_chars("abcbc".chars());
//! // => GeneralSAM<char>
//!
//! let state = sam.get_root_state();
//!
//! // "b" is not a suffix but at least a substring of "abcbc"
//! let state = state.feed_chars("b");
//! assert!(!state.is_accepting());
//!
//! // "bc" is a suffix of "abcbc"
//! let state = state.feed_chars("c");
//! assert!(state.is_accepting());
//!
//! // "bcbc" is a suffix of "abcbc"
//! let state = state.feed_chars("bc");
//! assert!(state.is_accepting());
//!
//! // "bcbcbc" is not a substring, much less a suffix of "abcbc"
//! let state = state.feed_chars("bc");
//! assert!(!state.is_accepting() && state.is_nil());
//! ```
//!
//! ```rust
//! # #[cfg(feature = "trie")] {
//! use general_sam::{sam::GeneralSAM, trie::Trie};
//!
//! let mut trie = Trie::default();
//! trie.insert_iter("hello".chars());
//! trie.insert_iter("Chielo".chars());
//!
//! let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());
//!
//! assert!(sam.get_root_state().feed_chars("lo").is_accepting());
//! assert!(sam.get_root_state().feed_chars("ello").is_accepting());
//! assert!(sam.get_root_state().feed_chars("elo").is_accepting());
//!
//! assert!(!sam.get_root_state().feed_chars("el").is_accepting());
//! assert!(!sam.get_root_state().feed_chars("el").is_nil());
//!
//! assert!(!sam.get_root_state().feed_chars("bye").is_accepting());
//! assert!(sam.get_root_state().feed_chars("bye").is_nil());
//! # }
//! ```
//!
//! # References
//!
//! - [Mehryar Mohri, Pedro Moreno, Eugene Weinstein.
//!   General suffix automaton construction algorithm and space bounds.][paper]
//! - 刘研绎《后缀自动机在字典树上的拓展》
//! - [广义后缀自动机 - OI Wiki][general-sam-oi-wiki]
//!
//! [paper]: https://doi.org/10.1016/j.tcs.2009.03.034
//! [general-sam-oi-wiki]: https://oi-wiki.org/string/general-sam/

pub mod sam;
pub mod trie_alike;

pub use {
    sam::{
        GeneralSAM, GeneralSAMNode, GeneralSAMNodeID, GeneralSAMState, SAM_NIL_NODE_ID,
        SAM_ROOT_NODE_ID,
    },
    trie_alike::{IterAsChain, TravelEvent, TrieNodeAlike},
};

#[cfg(feature = "trie")]
#[cfg_attr(docsrs, doc(cfg(feature = "trie")))]
pub mod trie;
#[cfg(feature = "trie")]
#[cfg_attr(docsrs, doc(cfg(feature = "trie")))]
pub use trie::{Trie, TrieNode, TrieNodeID, TrieState, TRIE_NIL_NODE_ID, TRIE_ROOT_NODE_ID};

#[cfg(test)]
mod tests;
