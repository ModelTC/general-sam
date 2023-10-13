use crate::{sam::GeneralSAM, trie::Trie};

#[test]
fn test_simple_bytes() {
    let sam = GeneralSAM::construct_from_bytes("abcbc".as_bytes().iter());
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
    let sam = GeneralSAM::construct_from_chars("abcbc".chars());
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
    let sam = GeneralSAM::construct_from_bytes("你好".as_bytes().iter());
    println!("sam: {:?}", sam);
    let state = sam.get_root_state();
    println!("state \"\": {:?}", state.node_id);
    let state = state.feed_bytes("你好");
    println!("state \"你好\": {:?}", state.node_id);
}

#[test]
fn test_chinese_chars() {
    let sam = GeneralSAM::construct_from_chars("你好".chars());
    println!("sam: {:?}", sam);
    let state = sam.get_root_state();
    println!("state \"\": {:?}", state.node_id);
    let state = state.feed_chars("你好");
    println!("state \"你好\": {:?}", state.node_id);
}

fn test_trie_suffix(vocab: &[&str]) {
    let mut trie = Trie::default();
    vocab.iter().for_each(|word| {
        trie.insert_iter(word.chars());
    });
    println!("trie: {:?}", trie);

    let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());
    println!("sam: {:?}", sam);
    vocab.iter().for_each(|word| {
        println!(
            "feed {}: {:?}",
            word,
            sam.get_root_state().feed_iter(word.chars()).node_id
        );
    });

    let is_suffix = |word_slice: &str| vocab.iter().any(|word| word.ends_with(word_slice));

    vocab.iter().for_each(|word| {
        word.char_indices().for_each(|(i, _)| {
            word.char_indices()
                .chain(Some((word.len(), '\0')))
                .for_each(|(j, _)| {
                    if i < j {
                        let state = sam.get_root_state().feed_iter(word[i..j].chars());
                        assert!(!state.is_nil());
                        println!(
                            "{}: {:?} {:?}",
                            word[i..j].to_owned(),
                            is_suffix(&word[i..j]),
                            state.is_accepting()
                        );
                        assert!(is_suffix(&word[i..j]) ^ !(state.is_accepting()));
                    }
                })
        });
    });

    println!(
        "topo order: {:?}",
        sam.get_topo_order()
            .map(|x| { x.node_id })
            .collect::<Vec<usize>>()
    );
    println!(
        "topo order rev: {:?}",
        sam.get_topo_order()
            .rev()
            .map(|x| { x.node_id })
            .collect::<Vec<usize>>()
    );
    assert!(sam
        .get_topo_order()
        .map(|x| { x.node_id })
        .collect::<Vec<usize>>()
        .iter()
        .rev()
        .zip(sam.get_topo_order().rev().map(|x| { x.node_id }))
        .all(|(x, y)| *x == y));
}

#[test]
fn test_chiense_trie_suffix() {
    let vocab = ["歌曲", "聆听歌曲", "播放歌曲", "歌词", "查看歌词"];
    test_trie_suffix(&vocab);
}

#[test]
fn test_simple_trie_suffix() {
    let vocab = ["ac", "bb", "b", "cc", "aabb", "a", "ba", "c", "aa"];
    test_trie_suffix(&vocab);
}
