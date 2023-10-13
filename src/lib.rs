pub mod sam;
pub mod trie;
pub mod trie_alike;

#[cfg(test)]
mod tests {
    use crate::sam;

    #[test]
    fn test_simple_bytes() {
        let sam = sam::GeneralSAM::construct_from_bytes("abcbc".as_bytes().iter());
        println!("sam: {:?}", sam);
        let state = sam.get_root_state();
        println!("state \"\": {:?}", state.node_id);
        let state = state.feed_bytes("bc");
        println!("state \"bc\": {:?}", state.node_id);
        let state = state.feed_bytes("b");
        println!("state \"bcb\": {:?}", state.node_id);
        let state = state.feed_bytes("c");
        println!("state \"bcbc\": {:?}", state.node_id);
        let state = state.feed_bytes("a");
        println!("state \"bcbca\": {:?}", state.node_id);
        let state = state.feed_bytes("a");
        println!("state \"bcbcaa\": {:?}", state.node_id);
    }

    #[test]
    fn test_simple_chars() {
        let sam = sam::GeneralSAM::construct_from_chars("abcbc".chars());
        println!("sam: {:?}", sam);
        let state = sam.get_root_state();
        println!("state \"\": {:?}", state.node_id);
        let state = state.feed_chars("bc");
        println!("state \"bc\": {:?}", state.node_id);
        let state = state.feed_chars("b");
        println!("state \"bcb\": {:?}", state.node_id);
        let state = state.feed_chars("c");
        println!("state \"bcbc\": {:?}", state.node_id);
        let state = state.feed_chars("a");
        println!("state \"bcbca\": {:?}", state.node_id);
        let state = state.feed_chars("a");
        println!("state \"bcbcaa\": {:?}", state.node_id);
    }

    #[test]
    fn test_chinese_bytes() {
        let sam = sam::GeneralSAM::construct_from_bytes("你好".as_bytes().iter());
        println!("sam: {:?}", sam);
        let state = sam.get_root_state();
        println!("state \"\": {:?}", state.node_id);
        let state = state.feed_bytes("你好");
        println!("state \"你好\": {:?}", state.node_id);
    }

    #[test]
    fn test_chinese_chars() {
        let sam = sam::GeneralSAM::construct_from_chars("你好".chars());
        println!("sam: {:?}", sam);
        let state = sam.get_root_state();
        println!("state \"\": {:?}", state.node_id);
        let state = state.feed_chars("你好");
        println!("state \"你好\": {:?}", state.node_id);
    }
}
