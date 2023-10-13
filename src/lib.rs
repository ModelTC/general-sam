pub mod sam;
pub mod trie_alike;

#[cfg(test)]
mod tests {
    use crate::sam;

    #[test]
    fn test_simple_str() {
        let sam = sam::GeneralSAM::construct_from_str("abcbc");
        println!("sam: {:?}", sam);
        let state = sam.get_root_state();
        println!("state \"\": {:?}", state.node_id);
        let state = state.feed_str("bc");
        println!("state \"bc\": {:?}", state.node_id);
        let state = state.feed_str("b");
        println!("state \"bcbc\": {:?}", state.node_id);
        let state = state.feed_str("c");
        println!("state \"bcbc\": {:?}", state.node_id);
        let state = state.feed_str("a");
        println!("state \"bcbca\": {:?}", state.node_id);
        let state = state.feed_str("a");
        println!("state \"bcbcaa\": {:?}", state.node_id);
    }
}
