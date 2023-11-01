//! Greedy tokenizer.

use std::ops::{AddAssign, SubAssign};

use crate::{GeneralSAM, GeneralSAMState, TrieNodeAlike};

use super::suffixwise::SuffixInTrieData;

pub struct GreedyTokenizer<'s, T: Ord + Clone, TokenIDType: Clone + Default> {
    sam: &'s GeneralSAM<T>,
    suffix_data: Vec<SuffixInTrieData<TokenIDType>>,
}

impl<'s, T: Ord + Clone, TokenIDType: Clone + Default + Eq> GreedyTokenizer<'s, T, TokenIDType> {
    pub fn build<TN: TrieNodeAlike<InnerType = T> + Clone, F: FnMut(&TN) -> TokenIDType>(
        sam: &'s GeneralSAM<T>,
        trie_node: TN,
        f: F,
    ) -> Self {
        Self {
            sam,
            suffix_data: SuffixInTrieData::build(sam, trie_node, f),
        }
    }

    pub fn tokenize<Iter: Iterator<Item = T>>(
        &self,
        iter: Iter,
        unk_token_id: &TokenIDType,
    ) -> Vec<(TokenIDType, usize)> {
        let mut res = Vec::new();

        let mut cur_state = self.sam.get_root_state();
        let mut cur_len = 0;

        let goto = |cur_len: &mut usize, cur_state: &mut GeneralSAMState<T>, key: &T| {
            cur_state.goto(key);
            cur_len.add_assign(1);
        };

        let push = |res: &mut Vec<_>, token_id: TokenIDType, token_len: usize| {
            if let Some((last_token_id, last_token_len)) = res.last_mut() {
                if *last_token_id == *unk_token_id && token_id == *unk_token_id {
                    *last_token_len += token_len;
                    return;
                }
            }
            res.push((token_id, token_len))
        };

        let pop_buffer =
            |cur_len: &mut usize, cur_state: &mut GeneralSAMState<T>, res: &mut Vec<_>| {
                let inner_data = self.suffix_data[cur_state.node_id]
                    .get(*cur_len)
                    .expect("invalid state");

                // TODO: optimize for unknown token:
                //       find the lower bound position where the suffix is prefixed with a token
                let (token_id, token_len) = inner_data.as_ref().map_or_else(
                    || (unk_token_id, 1),
                    |token_info| (&token_info.digested_trie_node, token_info.seq_len),
                );

                cur_len.sub_assign(token_len);
                push(res, token_id.clone(), token_len);

                while *cur_len < self.suffix_data[cur_state.node_id].get_min_suf_len() {
                    cur_state.goto_suffix_parent();
                }
            };

        for key in iter {
            debug_assert!(!cur_state.is_nil());
            while cur_len > 0 && !cur_state.has_trans(&key) {
                pop_buffer(&mut cur_len, &mut cur_state, &mut res);
            }
            if !cur_state.has_trans(&key) {
                debug_assert!(cur_state.is_root());
                push(&mut res, unk_token_id.clone(), 1);
                continue;
            }
            goto(&mut cur_len, &mut cur_state, &key);
        }

        while cur_len > 0 {
            pop_buffer(&mut cur_len, &mut cur_state, &mut res);
        }

        res
    }
}

#[cfg(feature = "trie")]
pub mod trie {
    use crate::{GeneralSAM, Trie, TrieNodeAlike, TrieNodeID, TrieState};

    impl<'s, T: Ord + Clone> super::GreedyTokenizer<'s, T, TrieNodeID> {
        pub fn build_from_trie(sam: &'s GeneralSAM<T>, trie_state: TrieState<'s, T>) -> Self {
            Self::build(sam, trie_state, |tn| tn.node_id)
        }
    }

    pub fn greedy_tokenize_with_trie<T: Ord + Clone, Iter: Iterator<Item = T>>(
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
}
