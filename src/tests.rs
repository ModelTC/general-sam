use crate::sam::GeneralSAM;

#[test]
fn test_example_from_chars() {
    let sam_from_chars = GeneralSAM::construct_from_chars("abcbc".chars());
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
    let sam_from_bytes = GeneralSAM::construct_from_bytes("abcbc");
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

#[cfg(feature = "trie")]
mod trie {
    use rand::{
        distributions::{Alphanumeric, DistString},
        rngs::StdRng,
        Rng, SeedableRng,
    };

    use crate::{GeneralSAM, Trie, SAM_ROOT_NODE_ID};

    #[test]
    fn test_example_from_trie() {
        let mut trie = Trie::default();

        trie.insert_iter("hello".chars());
        trie.insert_iter("Chielo".chars());

        let sam_from_trie: GeneralSAM<char> =
            GeneralSAM::construct_from_trie(trie.get_root_state());

        let state = sam_from_trie.get_root_state();
        assert!(state.is_root());
        let state = state.feed_chars("l");
        assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
        let state = state.feed_chars("o");
        assert!(state.is_accepting() && !state.is_nil() && !state.is_root());

        let state = sam_from_trie.get_root_state();
        assert!(state.is_root());
        let state = state.feed_chars("Chie");
        assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
        let state = state.feed_chars("lo");
        assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
    }

    #[cfg(feature = "trie")]
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
    }

    #[cfg(feature = "trie")]
    #[test]
    fn test_chiense_trie_suffix() {
        let vocab = ["歌曲", "聆听歌曲", "播放歌曲", "歌词", "查看歌词"];
        test_trie_suffix(&vocab);
    }

    #[cfg(feature = "trie")]
    #[test]
    fn test_simple_trie_suffix() {
        let vocab = ["ac", "bb", "b", "cc", "aabb", "a", "ba", "c", "aa"];
        test_trie_suffix(&vocab);
    }

    #[cfg(feature = "trie")]
    #[test]
    fn test_topo_and_suf_len_sorted_order() {
        let mut rng = StdRng::seed_from_u64(1134759173975);
        for _ in 0..10000 {
            let mut trie = Trie::default();
            for _ in 0..rng.gen_range(0..32) {
                let len = rng.gen_range(0..9);
                let string = Alphanumeric.sample_string(&mut rng, len);
                trie.insert_ref_iter(string.as_bytes().iter());
            }

            let sam: GeneralSAM<u8> = GeneralSAM::construct_from_trie(trie.get_root_state());

            let order = sam.get_topo_and_suf_len_sorted_node_ids();
            let rank = {
                let mut rank = vec![0; sam.num_of_nodes()];
                order.iter().enumerate().for_each(|(k, i)| {
                    rank[*i] = k;
                });
                rank
            };

            // verify that max suffix lengths should be sorted
            for pos in 0..order.len() - 1 {
                assert!(
                    sam.get_node(order[pos]).unwrap().max_suffix_len()
                        <= sam.get_node(order[pos + 1]).unwrap().max_suffix_len()
                );
            }

            // verify topological ordering
            order.iter().for_each(|node_id| {
                let node = sam.get_node(*node_id).unwrap();

                node.get_trans().values().for_each(|next_node_id| {
                    assert!(rank[*next_node_id] > rank[*node_id]);
                });
            });

            // verify suffix parent tree depth ordering
            order.iter().for_each(|node_id| {
                let node = sam.get_node(*node_id).unwrap();

                if *node_id != SAM_ROOT_NODE_ID {
                    assert!(rank[node.get_suffix_parent_id()] < rank[*node_id]);
                }
            });
        }
    }
}

#[cfg(all(feature = "utils", feature = "trie"))]
mod utils {
    use std::collections::BTreeMap;

    use crate::{
        utils::{
            rope::RopeBase,
            suffixwise::{SuffixInTrie, SuffixInTrieData},
            tokenize::GreedyTokenizer,
        },
        GeneralSAM, Trie,
    };

    #[test]
    fn test_suffix_in_trie_data() {
        let vocab = [
            "a", "ab", "b", "bc", "c", "d", "e", "f", "cd", "abcde", "你好", "🧡",
        ];
        let mut trie = Trie::default();
        let mut id_to_word = BTreeMap::new();
        for word in vocab {
            id_to_word.insert(trie.insert_iter(word.chars()), word);
        }

        let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());

        let data = SuffixInTrieData::build(&sam, trie.get_root_state(), |tn| tn.clone());
        for i in data.iter().skip(1) {
            let mut suffix_info = Vec::new();
            i.get_rope().for_each(|x| {
                suffix_info.push(x.into_inner().map(|x| {
                    let SuffixInTrie {
                        digested_trie_node: trie_node,
                        seq_len: chars_count,
                    } = x;
                    let word = id_to_word.get(&trie_node.node_id).unwrap();
                    assert_eq!(chars_count, word.chars().count());
                    (chars_count, word)
                }))
            });
            assert_eq!(
                suffix_info.len(),
                i.get_max_suf_len() - i.get_min_suf_len() + 1
            );
        }
    }

    #[test]
    fn test_tokenizer_simple() {
        let vocab = [
            "a", "ab", "b", "bc", "c", "d", "e", "f", "cd", "abcde", "你好", "🧡",
        ];
        let mut trie = Trie::default();
        let mut id_to_word = BTreeMap::new();
        for word in vocab {
            id_to_word.insert(trie.insert_iter(word.chars()), word);
        }

        let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());

        let tokenizer = GreedyTokenizer::build_from_trie(&sam, trie.get_root_state());

        dbg!(tokenizer.tokenize("abcde".chars(), &trie.num_of_nodes()));
        dbg!(tokenizer.tokenize("abcdf".chars(), &trie.num_of_nodes()));
        dbg!(tokenizer.tokenize("abca".chars(), &trie.num_of_nodes()));
        dbg!(tokenizer.tokenize("Hi，你好吗？".chars(), &trie.num_of_nodes()));
        dbg!(tokenizer.tokenize("🧡🧡🧡🧡🧡！".chars(), &trie.num_of_nodes()));
        dbg!(tokenizer.tokenize("abc".chars(), &trie.num_of_nodes()));
    }
}
