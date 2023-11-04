use crate::{GeneralSAM, BTreeTransTable};

#[cfg(feature = "utils")]
mod utils;

#[cfg(feature = "trie")]
mod trie;

#[test]
fn test_example_from_chars() {
    let sam_from_chars = GeneralSAM::<BTreeTransTable<char>>::from_chars("abcbc".chars());
    // => GeneralSAM<char>

    let state = sam_from_chars.get_root_state();
    assert!(state.is_root());
    let state = state.feed_chars("b");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("c");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("bc");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("bc");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
}

#[test]
fn test_example_from_bytes() {
    let sam_from_bytes = GeneralSAM::<BTreeTransTable<u8>>::from_bytes("abcbc");
    // => GeneralSAM<u8>

    let state = sam_from_bytes.get_root_state();
    assert!(state.is_root());
    let state = state.feed_bytes("b");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("c");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("bc");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("bc");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
}

#[test]
fn test_simple_bytes() {
    let sam = GeneralSAM::<BTreeTransTable<u8>>::from_bytes("abcbc".as_bytes().iter());
    let state = sam.get_root_state();
    assert!(!state.is_accepting() && !state.is_nil() && state.is_root());
    let state = state.feed_bytes("bc");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("b");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("c");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_bytes("a");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
    let state = state.feed_bytes("a");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
}

#[test]
fn test_simple_chars() {
    let sam = GeneralSAM::<BTreeTransTable<char>>::from_chars("abcbc".chars());
    let state = sam.get_root_state();
    assert!(!state.is_accepting() && !state.is_nil() && state.is_root());
    let state = state.feed_chars("bc");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("b");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("c");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("a");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
    let state = state.feed_chars("a");
    assert!(!state.is_accepting() && state.is_nil() && !state.is_root());
}

#[test]
fn test_chinese_bytes() {
    let sam = GeneralSAM::<BTreeTransTable<u8>>::from_bytes("你好".as_bytes().iter());
    let state = sam.get_root_state();
    assert!(!state.is_accepting() && !state.is_nil() && state.is_root());
    let state = state.feed_bytes("你好");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
}

#[test]
fn test_chinese_chars() {
    let sam = GeneralSAM::<BTreeTransTable<char>>::from_chars("你好".chars());
    let state = sam.get_root_state();
    assert!(!state.is_accepting() && !state.is_nil() && state.is_root());
    let state = state.feed_chars("你好");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
}
