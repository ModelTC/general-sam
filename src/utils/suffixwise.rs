//! Utilities to store suffix-wise data in a suffix automaton.

use std::{collections::LinkedList, convert::Infallible, ops::Deref};

use crate::{
    rope::{Rope, RopeBase, RopeData, RopeUntaggedInner, TreapBasedRopeBase},
    GeneralSam, GeneralSamState, TransitionTable, TravelEvent, TrieNodeAlike, SAM_NIL_NODE_ID,
    SAM_ROOT_NODE_ID,
};

#[derive(Clone, Default, Debug)]
pub struct SuffixwiseData<Inner: RopeData + Default> {
    data: Rope<Inner>,
    min_suf_len: usize,
    max_suf_len: usize,
}

impl<Inner: RopeData + Default> SuffixwiseData<Inner> {
    pub fn get_rope(&self) -> &Rope<Inner> {
        &self.data
    }

    pub fn get_min_suf_len(&self) -> usize {
        self.min_suf_len
    }

    pub fn get_max_suf_len(&self) -> usize {
        self.max_suf_len
    }

    pub fn map<NewInner: RopeData + Default, F: FnOnce(&Rope<Inner>) -> Rope<NewInner>>(
        &self,
        f: F,
    ) -> SuffixwiseData<NewInner> {
        SuffixwiseData {
            data: f(&self.data),
            min_suf_len: self.min_suf_len,
            max_suf_len: self.max_suf_len,
        }
    }

    pub fn get(&self, suf_len: usize) -> Option<Inner> {
        if self.data.is_empty()
            || self.max_suf_len == 0
            || self.min_suf_len == 0
            || suf_len < self.min_suf_len
            || suf_len > self.max_suf_len
        {
            return None;
        }
        Some(
            self.data
                .query(suf_len - self.min_suf_len)
                .expect("invalid suffixwise data")
                .as_ref()
                .deref()
                .to_owned(),
        )
    }

    pub fn build_from_sam<
        TransTable: TransitionTable,
        Iter: IntoIterator<Item = (usize, Inner)>,
        FInit: FnMut(usize) -> Iter,
    >(
        sam: &GeneralSam<TransTable>,
        mut f_init: FInit,
    ) -> Vec<Self> {
        let mut res = vec![Self::default(); sam.num_of_nodes()];
        for node_id in sam.get_topo_and_suf_len_sorted_node_ids().iter().copied() {
            assert_ne!(node_id, SAM_NIL_NODE_ID);

            let node = sam.get_node(node_id).expect("invalid GeneralSam");
            let node_data = res
                .get_mut(node_id)
                .unwrap_or_else(|| panic!("invalid node id: {}", node_id));

            node_data.max_suf_len = node.max_suffix_len();

            if node_id == SAM_ROOT_NODE_ID {
                node_data.min_suf_len = 0;

                node_data.data = Rope::new(Inner::default());
            } else {
                let parent_id = node.get_suffix_parent_id();
                let parent = sam.get_node(parent_id).expect("invalid GeneralSam");

                node_data.min_suf_len = parent.max_suffix_len() + 1;

                assert_eq!(
                    node_data.data.len(),
                    node_data.max_suf_len - node_data.min_suf_len + 1
                );

                for (len, data) in f_init(node_id) {
                    assert!(len >= node_data.min_suf_len && len <= node_data.max_suf_len);
                    let (left, right) = node_data.data.split(len - node_data.min_suf_len);
                    let (_, right) = right.split(1);
                    node_data.data = left.merge(&Rope::new(data)).merge(&right);
                }

                assert_eq!(
                    node_data.data.len(),
                    node_data.max_suf_len - node_data.min_suf_len + 1
                );
            }

            node.get_trans()
                .transitions()
                .copied()
                .for_each(|target_id| {
                    res[target_id].data = res[target_id].data.merge(&res[node_id].data)
                });
        }
        res
    }
}

#[derive(Clone, Debug)]
pub struct SuffixInTrie<Digested: Clone> {
    pub digested_trie_node: Digested,
    pub seq_len: usize,
}

pub type UntaggedSuffixData<Inner> = SuffixwiseData<RopeUntaggedInner<Inner>>;
pub type SuffixInTrieData<D> = UntaggedSuffixData<Option<SuffixInTrie<D>>>;

impl<Digested: Clone> SuffixInTrieData<Digested> {
    pub fn build<
        TransTable: TransitionTable,
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        F: FnMut(&TN) -> Digested,
    >(
        sam: &GeneralSam<TransTable>,
        trie_node: TN,
        mut f: F,
    ) -> Vec<Self> {
        let mut sam_to_data = vec![LinkedList::<SuffixInTrie<Digested>>::new(); sam.num_of_nodes()];
        let callback =
            |event: TravelEvent<(&GeneralSamState<_, &GeneralSam<_>>, &TN), _, _>| -> Result<_, Infallible> {
                match event {
                    crate::TravelEvent::Pop((sam_state, trie_state), len) => {
                        if trie_state.is_accepting() {
                            sam_to_data[sam_state.node_id].push_back(SuffixInTrie {
                                digested_trie_node: f(trie_state),
                                seq_len: len,
                            });
                        }
                        Ok(len)
                    }
                    crate::TravelEvent::PushRoot(_) => Ok(0),
                    crate::TravelEvent::Push(_, len, _) => Ok(len + 1),
                }
            };
        sam.get_root_state().bfs_along(trie_node, callback).unwrap();
        Self::build_from_sam(sam, |node_id| {
            sam_to_data[node_id]
                .iter()
                .map(|x| (x.seq_len, Some(x.clone()).into()))
        })
    }
}
