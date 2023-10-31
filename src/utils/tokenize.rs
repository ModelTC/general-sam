use std::ops::{AddAssign, SubAssign};

use crate::{GeneralSAM, GeneralSAMState, TrieNodeAlike};

use super::suffixwise::SuffixInTrieData;

pub struct GreedyTokenizer<'s, T: Ord + Clone, TokenIDType: Clone + Default> {
    sam: &'s GeneralSAM<T>,
    suffix_data: Vec<SuffixInTrieData<TokenIDType>>,
}

impl<'s, T: Ord + Clone, TokenIDType: Clone + Default + std::fmt::Debug>
    GreedyTokenizer<'s, T, TokenIDType>
{
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

        let pop = |cur_len: &mut usize, cur_state: &mut GeneralSAMState<T>, res: &mut Vec<_>| {
            let inner_data = self.suffix_data[cur_state.node_id]
                .get(*cur_len)
                .expect("invalid state");

            let (token_id, token_len) = inner_data.as_ref().map_or_else(
                || (unk_token_id, *cur_len),
                |token_info| (&token_info.digested_trie_node, token_info.seq_len),
            );

            cur_len.sub_assign(token_len);
            res.push((token_id.clone(), token_len));

            while *cur_len < self.suffix_data[cur_state.node_id].get_min_suf_len() {
                cur_state.goto_suffix_parent();
            }
        };

        for key in iter {
            debug_assert!(!cur_state.is_nil());
            while cur_len > 0 && !cur_state.has_trans(&key) {
                pop(&mut cur_len, &mut cur_state, &mut res);
            }
            if !cur_state.has_trans(&key) {
                cur_state = self.sam.get_root_state();
                res.push((unk_token_id.clone(), 1));
                continue;
            }
            goto(&mut cur_len, &mut cur_state, &key);
        }

        while cur_len > 0 {
            pop(&mut cur_len, &mut cur_state, &mut res);
        }

        res
    }
}

#[cfg(feature = "trie")]
pub mod trie {
    use crate::{GeneralSAM, TrieNodeID, TrieState};

    impl<'s, T: Ord + Clone> super::GreedyTokenizer<'s, T, TrieNodeID> {
        pub fn build_from_trie(sam: &'s GeneralSAM<T>, trie_state: TrieState<'s, T>) -> Self {
            Self::build(sam, trie_state, |tn| tn.node_id)
        }
    }
}
