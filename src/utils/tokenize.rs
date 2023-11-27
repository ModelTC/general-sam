//! Greedy tokenizer.

use std::ops::{AddAssign, Deref, SubAssign};

use crate::{GeneralSAM, GeneralSAMState, TransitionTable, TrieNodeAlike};

use super::suffixwise::SuffixInTrieData;

/// Greedy tokenizer with a general suffix automaton of the vocabulary.
///
/// Assuming that the input length is $n$, the maximum word length is $l$,
/// and querying transitions in the trie takes $\mathcal{O}\left(\log{\Sigma}\right)$ time,
/// then the overall time complexity of this implementation is
/// $\mathcal{O}\left( n \cdot \left( \log{l} + \log{\Sigma} \right) \right)$.
///
/// The main optimization is to store suffix-wise information with persistent ropes.
/// For each suffix in a state of the suffix automaton,
/// the longest word matching the prefix of the suffix is stored in the rope.
/// And the information stored in a state
/// will be further merged in the ropes of its successors.
#[derive(Clone, Debug)]
pub struct GreedyTokenizer<
    TransTable: TransitionTable,
    TokenIDType: Clone + Default + PartialEq,
    SAMRef: Deref<Target = GeneralSAM<TransTable>>,
> {
    sam: SAMRef,
    suffix_data: Vec<SuffixInTrieData<TokenIDType>>,
}

pub struct OwnedGeneralSAM<TransTable: TransitionTable> {
    pub sam: GeneralSAM<TransTable>,
}

impl<TransTable: TransitionTable> Deref for OwnedGeneralSAM<TransTable> {
    type Target = GeneralSAM<TransTable>;

    fn deref(&self) -> &Self::Target {
        &self.sam
    }
}

impl<TransTable: TransitionTable, TokenIDType: Clone + Default + PartialEq>
    GreedyTokenizer<TransTable, TokenIDType, OwnedGeneralSAM<TransTable>>
{
    pub fn build_from_sam<
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        F: FnMut(&TN) -> TokenIDType,
    >(
        sam: GeneralSAM<TransTable>,
        trie_node: TN,
        f: F,
    ) -> Self {
        Self {
            suffix_data: SuffixInTrieData::build(&sam, trie_node, f),
            sam: OwnedGeneralSAM { sam },
        }
    }
}

impl<
        TransTable: TransitionTable,
        TokenIDType: Clone + Default + PartialEq,
        SAMRef: Deref<Target = GeneralSAM<TransTable>>,
    > GreedyTokenizer<TransTable, TokenIDType, SAMRef>
{
    pub fn get_sam(&self) -> &SAMRef {
        &self.sam
    }

    pub fn get_sam_ref(&self) -> &GeneralSAM<TransTable> {
        &self.sam
    }

    pub fn get_suffix_data(&self) -> &Vec<SuffixInTrieData<TokenIDType>> {
        &self.suffix_data
    }

    pub fn inner_as_ref(
        &self,
    ) -> GreedyTokenizer<TransTable, TokenIDType, &GeneralSAM<TransTable>> {
        GreedyTokenizer {
            sam: &self.sam,
            suffix_data: self.suffix_data.clone(),
        }
    }

    pub fn build<
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        F: FnMut(&TN) -> TokenIDType,
    >(
        sam: SAMRef,
        trie_node: TN,
        f: F,
    ) -> Self {
        Self {
            suffix_data: SuffixInTrieData::build(sam.deref(), trie_node, f),
            sam,
        }
    }

    pub fn tokenize<Iter: Iterator<Item = TransTable::KeyType>>(
        &self,
        iter: Iter,
        unk_token_id: &TokenIDType,
    ) -> Vec<(TokenIDType, usize)> {
        let mut res = Vec::new();

        let push = |res: &mut Vec<_>, token_id: TokenIDType, token_len: usize| {
            if let Some((last_token_id, last_token_len)) = res.last_mut() {
                if *last_token_id == *unk_token_id && token_id == *unk_token_id {
                    *last_token_len += token_len;
                    return;
                }
            }
            res.push((token_id, token_len))
        };

        let pop_buffer = |cur_len: &mut usize,
                          cur_state: &mut GeneralSAMState<TransTable, &GeneralSAM<TransTable>>,
                          res: &mut Vec<_>| {
            let inner_data = self.suffix_data[cur_state.node_id]
                .get(*cur_len)
                .expect("invalid state");

            // TODO: Optimize for unknown tokens:
            // Find the lower bound position where the suffix is prefixed with a token.
            // But this does not improve the time complexity, pending...
            let (token_id, token_len) = inner_data.as_ref().map_or_else(
                || (unk_token_id, 1),
                |token_info| (&token_info.digested_trie_node, token_info.seq_len),
            );

            cur_len.sub_assign(token_len);
            push(res, token_id.clone(), token_len);
        };

        let mut cur_state = self.sam.get_root_state();
        let mut cur_len = 0;

        for key in iter {
            debug_assert!(!cur_state.is_nil());
            let mut nxt_state = cur_state.get_non_nil_trans(&key);
            while cur_len > 0 && nxt_state.is_none() {
                pop_buffer(&mut cur_len, &mut cur_state, &mut res);

                if cur_len < self.suffix_data[cur_state.node_id].get_min_suf_len() {
                    while cur_len < self.suffix_data[cur_state.node_id].get_min_suf_len() {
                        cur_state.goto_suffix_parent();
                    }
                    nxt_state = cur_state.get_non_nil_trans(&key);
                }
            }
            if let Some(nxt) = nxt_state {
                cur_state = nxt;
                cur_len.add_assign(1);
            } else {
                debug_assert!(cur_state.is_root());
                push(&mut res, unk_token_id.clone(), 1);
            }
        }

        while cur_len > 0 {
            pop_buffer(&mut cur_len, &mut cur_state, &mut res);

            while cur_len < self.suffix_data[cur_state.node_id].get_min_suf_len() {
                cur_state.goto_suffix_parent();
            }
        }

        res
    }
}

#[cfg(feature = "trie")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "trie")))]
pub mod trie {
    use std::ops::Deref;

    use crate::{GeneralSAM, TransitionTable, Trie, TrieNodeAlike, TrieNodeID, TrieState};

    use super::OwnedGeneralSAM;

    impl<TransTable: TransitionTable, SAMRef: Deref<Target = GeneralSAM<TransTable>>>
        super::GreedyTokenizer<TransTable, TrieNodeID, SAMRef>
    {
        pub fn build_from_trie<TT: TransitionTable<KeyType = TransTable::KeyType>>(
            sam: SAMRef,
            trie_state: TrieState<TT, &Trie<TT>>,
        ) -> Self {
            Self::build(sam, trie_state, |tn| tn.node_id)
        }
    }

    impl<TransTable: TransitionTable>
        super::GreedyTokenizer<TransTable, TrieNodeID, OwnedGeneralSAM<TransTable>>
    {
        pub fn build_from_sam_and_trie<TT: TransitionTable<KeyType = TransTable::KeyType>>(
            sam: GeneralSAM<TransTable>,
            trie_state: TrieState<TT, &Trie<TT>>,
        ) -> Self {
            Self::build_from_sam(sam, trie_state, |tn| tn.node_id)
        }
    }

    /// Greedy tokenizer with a trie of the vocabulary.
    ///
    /// Assuming that the input length is $n$, the maximum word length is $l$,
    /// and querying transitions in the trie takes $\mathcal{O}\left(\log{\Sigma}\right)$ time,
    /// then the overall time complexity of this implementation is
    /// $\mathcal{O}\left( n \cdot l \cdot \log{\Sigma} \right)$.
    pub fn greedy_tokenize_with_trie<
        TransTable: TransitionTable,
        Iter: Iterator<Item = TransTable::KeyType>,
    >(
        trie: &Trie<TransTable>,
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
            for i in cur..seq.len() {
                if !cur_state.is_root() && cur_state.is_accepting() {
                    best = Some((cur_state.node_id, i - cur));
                }
                let key = &seq[i];
                cur_state.goto(key);
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
                push(&mut res, unk_token_id, 1);
                cur += 1;
            }
        }

        res
    }
}
