use std::{collections::LinkedList, convert::Infallible, ops::Deref};

use crate::{
    rope::{Rope, RopeBase, RopeData, RopeUntaggedInner, TreapBasedRopeBase},
    GeneralSAM, GeneralSAMState, TravelEvent, TrieNodeAlike, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID,
};

#[derive(Clone, Default, Debug)]
pub struct SuffixwiseData<Inner: RopeData + Default> {
    data: Rope<Inner>,
    min_suf_len: usize,
    max_suf_len: usize,
}

impl<Inner: RopeData + Default> SuffixwiseData<Inner> {
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
        T: Ord + Clone,
        Iter: Iterator<Item = (usize, Inner)>,
        FInit: FnMut(usize) -> Iter,
    >(
        sam: &GeneralSAM<T>,
        mut f_init: FInit,
    ) -> Vec<Self> {
        let mut res = vec![Self::default(); sam.num_of_nodes()];
        for node_id in sam.get_topo_and_suf_len_sorted_node_ids().iter().copied() {
            assert_ne!(node_id, SAM_NIL_NODE_ID);

            let node = sam.get_node(node_id).expect("invalid GeneralSAM");
            let node_data = res
                .get_mut(node_id)
                .unwrap_or_else(|| panic!("invalid node id: {}", node_id));

            node_data.max_suf_len = node.max_suffix_len();

            if node_id == SAM_ROOT_NODE_ID {
                node_data.min_suf_len = 0;

                node_data.data = Rope::new(Inner::default());
            } else {
                let parent_id = node.get_suffix_parent_id();
                let parent = sam.get_node(parent_id).expect("invalid GeneralSAM");

                node_data.min_suf_len = parent.max_suffix_len() + 1;
            }

            assert_eq!(
                node_data.data.len(),
                node_data.max_suf_len - node_data.min_suf_len + 1
            );

            for (len, data) in f_init(node_id) {
                let (left, right) = node_data.data.split(len - node_data.min_suf_len);
                let (_, right) = right.split(1);
                node_data.data = left.merge(&Rope::new(data)).merge(&right);
            }

            assert_eq!(
                node_data.data.len(),
                node_data.max_suf_len - node_data.min_suf_len + 1
            );

            node.get_trans().values().copied().for_each(|target_id| {
                res[target_id].data = res[target_id].data.merge(&res[node_id].data)
            });
        }
        res
    }
}

#[derive(Clone, Debug)]
pub struct SuffixInTrie<TN: TrieNodeAlike + Clone>
where
    TN::InnerType: Ord + Clone,
{
    pub trie_node: TN,
    pub seq_len: usize,
}

pub type SuffixInTrieData<TN> = SuffixwiseData<RopeUntaggedInner<Option<SuffixInTrie<TN>>>>;

impl<TN: TrieNodeAlike + Clone> SuffixInTrieData<TN>
where
    TN::InnerType: Ord + Clone,
{
    pub fn build(sam: &GeneralSAM<TN::InnerType>, trie_node: TN) -> Vec<Self> {
        let mut sam_to_data = vec![LinkedList::<SuffixInTrie<TN>>::new(); sam.num_of_nodes()];
        let callback = |event: TravelEvent<(&GeneralSAMState<TN::InnerType>, &TN), _, _>| -> Result<usize, Infallible> {
            match event {
                crate::TravelEvent::Pop((sam_state, trie_state), len) => {
                    if trie_state.is_accepting() {
                        sam_to_data[sam_state.node_id].push_back(SuffixInTrie {
                            trie_node: trie_state.clone(),
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

#[cfg(feature = "trie")]
#[test]
fn test_suffix_in_trie_data() {
    use std::collections::BTreeMap;

    use crate::trie::Trie;

    let vocab = ["a", "ab", "b", "bc", "c", "d", "e", "f", "cd", "abcde", "你好", "🧡"];
    let mut trie = Trie::default();
    let mut id_to_word = BTreeMap::new();
    for word in vocab {
        id_to_word.insert(trie.insert_iter(word.chars()), word);
    }

    let sam: GeneralSAM<char> = GeneralSAM::construct_from_trie(trie.get_root_state());
    let data = SuffixInTrieData::build(&sam, trie.get_root_state());
    for i in data {
        let mut suffix_info = Vec::new();
        i.data.for_each(|x| {
            suffix_info.push(x.into_inner().map(|x| {
                let SuffixInTrie { trie_node, seq_len: chars_count } = x;
                (chars_count, id_to_word.get(&trie_node.node_id).unwrap())
            }))
        });
        dbg!(i.min_suf_len);
        dbg!(i.max_suf_len);
        dbg!(suffix_info);
    }
}
