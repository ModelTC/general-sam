use crate::rope::{RopeBase, RopeUntaggedInner, TreapBasedRopeBase, UntaggedRope};

#[test]
fn test_rope() {
    let rope = UntaggedRope::<char>::default();
    assert!(rope.get(0).is_none());
    assert!(rope.is_empty());
    assert_eq!(rope.len(), 0);

    let rope = rope.push_front('a'.into());
    assert!(rope.get(0).is_some_and(|x| *x == 'a'));
    assert!(!rope.is_empty());
    assert_eq!(rope.len(), 1);

    let rope = rope.push_front('b'.into());
    let rope = rope.push_back('c'.into());
    assert!(!rope.is_empty());
    assert_eq!(rope.len(), 3);
    assert!(rope.get(0).is_some_and(|x| *x == 'b'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'c'));
    assert!(rope.get(3).is_none());
    assert!(rope.query(0).is_some_and(|x| ***x == 'b'));
    assert!(rope.query(1).is_some_and(|x| ***x == 'a'));
    assert!(rope.query(2).is_some_and(|x| ***x == 'c'));
    assert!(rope.query(3).is_none());

    let rope = rope.reverse();
    assert!(!rope.is_empty());
    assert_eq!(rope.len(), 3);
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

    use rand::{
        distributions::{Alphanumeric, DistString},
        rngs::StdRng,
        Rng, SeedableRng,
    };

    use crate::{
        tokenize::trie::greedy_tokenize_with_trie,
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

    fn case_tokenizer<T: Ord + Clone, Iter: Iterator<Item = T>>(
        tokenizer: &GreedyTokenizer<T, usize>,
        trie: &Trie<T>,
        seq: Iter,
    ) {
        let seq: Box<[_]> = seq.collect();
        let output = tokenizer.tokenize(seq.iter().cloned(), &trie.num_of_nodes());
        let expected = greedy_tokenize_with_trie(trie, seq.iter().cloned());
        output.iter().zip(expected.iter()).for_each(|(o, e)| {
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

    fn case_tokenizer_vocab<T: Ord + Clone, F: FnMut(String) -> Vec<T>>(
        vocab_size: usize,
        token_len: usize,
        seed: u64,
        mut f: F,
    ) {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut trie = Trie::default();
        for _ in 0..rng.gen_range(0..vocab_size) {
            let len = rng.gen_range(0..token_len);
            let string = Alphanumeric.sample_string(&mut rng, len);
            trie.insert_ref_iter(f(string).iter());
        }

        let sam: GeneralSAM<T> = GeneralSAM::construct_from_trie(trie.get_root_state());

        let tokenizer = GreedyTokenizer::build_from_trie(&sam, trie.get_root_state());

        for _ in 0..rng.gen_range(0..4096) {
            let len = rng.gen_range(0..1024);
            let string = Alphanumeric.sample_string(&mut rng, len);
            case_tokenizer(&tokenizer, &trie, f(string).iter().cloned());
        }
    }

    #[test]
    fn test_tokenizer_small_vocab_bytes() {
        case_tokenizer_vocab(10, 32, 2450679142816, |s| s.bytes().collect());
        case_tokenizer_vocab(10, 32, 1928750982347, |s| s.bytes().collect());
        case_tokenizer_vocab(10, 32, 9173459982325, |s| s.bytes().collect());
        case_tokenizer_vocab(10, 8, 2450679142816, |s| s.bytes().collect());
        case_tokenizer_vocab(10, 8, 1928750982347, |s| s.bytes().collect());
        case_tokenizer_vocab(10, 8, 9173459982325, |s| s.bytes().collect());
    }

    #[test]
    fn test_tokenizer_small_vocab_chars() {
        case_tokenizer_vocab(10, 32, 2450679142816, |s| s.chars().collect());
        case_tokenizer_vocab(10, 32, 1928750982347, |s| s.chars().collect());
        case_tokenizer_vocab(10, 32, 9173459982325, |s| s.chars().collect());
        case_tokenizer_vocab(10, 8, 2450679142816, |s| s.chars().collect());
        case_tokenizer_vocab(10, 8, 1928750982347, |s| s.chars().collect());
        case_tokenizer_vocab(10, 8, 9173459982325, |s| s.chars().collect());
    }

    #[test]
    fn test_tokenizer_large_vocab_bytes() {
        case_tokenizer_vocab(64000, 32, 1928750982347, |s| s.bytes().collect());
        case_tokenizer_vocab(64000, 32, 2450679142816, |s| s.bytes().collect());
        case_tokenizer_vocab(64000, 8, 1928750982347, |s| s.bytes().collect());
        case_tokenizer_vocab(64000, 8, 2450679142816, |s| s.bytes().collect());
        case_tokenizer_vocab(64000, 4, 1928750982347, |s| s.bytes().collect());
        case_tokenizer_vocab(64000, 4, 2450679142816, |s| s.bytes().collect());
    }

    #[test]
    fn test_tokenizer_large_vocab_chars() {
        case_tokenizer_vocab(64000, 32, 1928750982347, |s| s.chars().collect());
        case_tokenizer_vocab(64000, 32, 2450679142816, |s| s.chars().collect());
        case_tokenizer_vocab(64000, 8, 1928750982347, |s| s.chars().collect());
        case_tokenizer_vocab(64000, 8, 2450679142816, |s| s.chars().collect());
        case_tokenizer_vocab(64000, 4, 1928750982347, |s| s.chars().collect());
        case_tokenizer_vocab(64000, 4, 2450679142816, |s| s.chars().collect());
    }
}