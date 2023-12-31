use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    Rng, SeedableRng,
};

use crate::{BTreeTransTable, GeneralSAM, Trie, SAM_ROOT_NODE_ID};

#[test]
fn test_example_from_trie() {
    let mut trie = Trie::<BTreeTransTable<_>>::default();

    trie.insert_iter("hello".chars());
    trie.insert_iter("Chielo".chars());

    let sam = GeneralSAM::<BTreeTransTable<_>>::from_trie(trie.get_root_state());

    let state = sam.get_root_state();
    assert!(state.is_root());
    let state = state.feed_chars("l");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("o");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());

    let state = sam.get_root_state();
    assert!(state.is_root());
    let state = state.feed_chars("Chie");
    assert!(!state.is_accepting() && !state.is_nil() && !state.is_root());
    let state = state.feed_chars("lo");
    assert!(state.is_accepting() && !state.is_nil() && !state.is_root());
}

fn case_trie_suffix(vocab: &[&str]) {
    let mut trie = Trie::<BTreeTransTable<_>>::default();
    vocab.iter().for_each(|word| {
        trie.insert_iter(word.chars());
    });

    let sam = GeneralSAM::<BTreeTransTable<_>>::from_trie(trie.get_root_state());

    let is_suffix = |word_slice: &str| vocab.iter().any(|word| word.ends_with(word_slice));

    vocab.iter().for_each(|word| {
        word.char_indices().for_each(|(i, _)| {
            word.char_indices()
                .chain(Some((word.len(), '\0')))
                .for_each(|(j, _)| {
                    if i < j {
                        let state = sam.get_root_state().feed_iter(word[i..j].chars());
                        assert!(!state.is_nil());
                        assert!(is_suffix(&word[i..j]) ^ !(state.is_accepting()));
                    }
                })
        });
    });
}

#[test]
fn test_chiense_trie_suffix() {
    let vocab = ["歌曲", "聆听歌曲", "播放歌曲", "歌词", "查看歌词"];
    case_trie_suffix(&vocab);
}

#[test]
fn test_simple_trie_suffix() {
    let vocab = ["ac", "bb", "b", "cc", "aabb", "a", "ba", "c", "aa"];
    case_trie_suffix(&vocab);
}

#[test]
fn test_topo_and_suf_len_sorted_order() {
    let mut rng = StdRng::seed_from_u64(1134759173975);
    for _ in 0..10000 {
        let mut trie = Trie::<BTreeTransTable<_>>::default();
        for _ in 0..rng.gen_range(0..32) {
            let len = rng.gen_range(0..9);
            let string = Alphanumeric.sample_string(&mut rng, len);
            trie.insert_ref_iter(string.as_bytes().iter());
        }

        let sam = GeneralSAM::<BTreeTransTable<u8>>::from_trie(trie.get_root_state());

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
