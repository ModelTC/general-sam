use crate::rope::{RopeBase, RopeUntaggedInner, TreapBasedRopeBase, UntaggedRope};

#[test]
fn test_rope() {
    let rope = UntaggedRope::<char>::default();
    assert!(rope.get(0).is_none());

    let rope = rope.push_front('a'.into());
    assert!(rope.get(0).is_some_and(|x| *x == 'a'));

    let rope = rope.push_front('b'.into());
    let rope = rope.push_back('c'.into());
    assert!(rope.get(0).is_some_and(|x| *x == 'b'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'c'));
    assert!(rope.get(3).is_none());
    assert!(rope.query(0).is_some_and(|x| ***x == 'b'));
    assert!(rope.query(1).is_some_and(|x| ***x == 'a'));
    assert!(rope.query(2).is_some_and(|x| ***x == 'c'));
    assert!(rope.query(3).is_none());

    let rope = rope.reverse();
    assert!(rope.get(0).is_some_and(|x| *x == 'c'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'b'));
    assert!(rope.get(3).is_none());

    let (l, r) = rope.split(1);
    assert!(rope.get(0).is_some_and(|x| *x == 'c'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'b'));
    assert!(rope.get(3).is_none());
    assert!(l.get(0).is_some_and(|x| *x == 'c'));
    assert!(l.get(1).is_none());
    assert!(r.get(0).is_some_and(|x| *x == 'a'));
    assert!(r.get(1).is_some_and(|x| *x == 'b'));
    assert!(r.get(2).is_none());

    let to_vec = |p: UntaggedRope<char>| -> Vec<char> {
        let mut res = Vec::<char>::new();
        p.for_each(&mut |d: RopeUntaggedInner<char>| res.push(*d));
        res
    };

    let reversed = rope.reverse();
    let v = to_vec(reversed);
    v.iter()
        .zip(['b', 'a', 'c'])
        .for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(rope);
    v.iter()
        .zip(['c', 'a', 'b'])
        .for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(l);
    v.iter().zip(['c']).for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(r);
    v.iter()
        .zip(['a', 'b'])
        .for_each(|(i, j)| assert_eq!(*i, j));
}

#[cfg(feature = "trie")]
mod trie {
    use std::collections::BTreeMap;

    use crate::{
        utils::{
            rope::RopeBase,
            suffixwise::{SuffixInTrie, SuffixInTrieData},
            tokenize::GreedyTokenizer,
        },
        GeneralSAM, Trie, TrieNodeAlike,
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

    fn greedy_tokenize_with_trie<T: Ord + Clone, Iter: Iterator<Item = T>>(
        trie: &Trie<T>,
        seq: Iter,
    ) -> Vec<(usize, usize)> {
        let unk_token_id = trie.num_of_nodes();

        let mut res = Vec::new();

        let push = |res: &mut Vec<_>, token_id: usize, token_len: usize| {
            if let Some((last_token_id, last_token_len)) = res.last_mut() {
                if *last_token_id == unk_token_id && token_id == unk_token_id {
                    *last_token_len += token_len;
                    return;
                }
            }
            res.push((token_id, token_len))
        };

        let seq: Box<[_]> = seq.collect();
        let mut cur = 0;
        while cur < seq.len() {
            let mut best: Option<(usize, usize)> = None;
            let mut cur_state = trie.get_root_state();
            let mut cur_len = 0;
            for i in cur..seq.len() {
                if !cur_state.is_root() && cur_state.is_accepting() {
                    best = Some((cur_state.node_id, i - cur));
                }
                let key = &seq[i];
                cur_state.goto(key);
                cur_len += 1;
                if cur_state.is_nil() {
                    break;
                }
            }
            if !cur_state.is_root() && !cur_state.is_nil() && cur_state.is_accepting() {
                best = Some((cur_state.node_id, seq.len() - cur));
            }
            if let Some((best_token_id, best_token_len)) = best {
                push(&mut res, best_token_id, best_token_len);
                cur += best_token_len;
            } else {
                let chunk_size = (cur_len - 1).max(1);
                push(&mut res, unk_token_id, chunk_size);
                cur += chunk_size;
            }
        }

        res
    }

    fn case_tokenizer<T: Ord + Clone + std::fmt::Debug, Iter: Iterator<Item = T>>(
        tokenizer: &GreedyTokenizer<T, usize>,
        trie: &Trie<T>,
        seq: Iter,
    ) {
        let seq: Box<[_]> = seq.collect();
        let output = tokenizer.tokenize(seq.iter().cloned(), &trie.num_of_nodes());
        let expected = greedy_tokenize_with_trie(trie, seq.iter().cloned());
        dbg!(&seq);
        output.iter().zip(expected.iter()).for_each(|(o, e)| {
            dbg!(&o, &e);
            assert_eq!(*o, *e);
        });
    }

    #[test]
    fn test_tokenizer_simple_chars() {
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

        case_tokenizer(&tokenizer, &trie, "abcde".chars());
        case_tokenizer(&tokenizer, &trie, "abcdf".chars());
        case_tokenizer(&tokenizer, &trie, "abca".chars());
        case_tokenizer(&tokenizer, &trie, "Hi，你好吗？".chars());
        case_tokenizer(&tokenizer, &trie, "🧡🧡🧡🧡🧡！".chars());
        case_tokenizer(&tokenizer, &trie, "abc".chars());
    }

    #[test]
    fn test_tokenizer_simple_bytes() {
        let vocab = [
            "a", "ab", "b", "bc", "c", "d", "e", "f", "cd", "abcde", "你好", "🧡",
        ];
        let mut trie = Trie::default();
        let mut id_to_word = BTreeMap::new();
        for word in vocab {
            id_to_word.insert(trie.insert_iter(word.bytes()), word);
        }

        let sam: GeneralSAM<u8> = GeneralSAM::construct_from_trie(trie.get_root_state());

        let tokenizer = GreedyTokenizer::build_from_trie(&sam, trie.get_root_state());

        case_tokenizer(&tokenizer, &trie, "abcde".bytes());
        case_tokenizer(&tokenizer, &trie, "abcdf".bytes());
        case_tokenizer(&tokenizer, &trie, "abca".bytes());
        case_tokenizer(&tokenizer, &trie, "Hi，你好吗？".bytes());
        case_tokenizer(&tokenizer, &trie, "🧡🧡🧡🧡🧡！".bytes());
        case_tokenizer(&tokenizer, &trie, "abc".bytes());
    }
}
