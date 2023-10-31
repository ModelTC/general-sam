//! States of a general suffix automaton.

use crate::trie_alike::{TravelEvent, TrieNodeAlike};

use super::{GeneralSAM, GeneralSAMNode, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID};

#[derive(Debug, Clone)]
pub struct GeneralSAMState<'s, T: Ord + Clone> {
    pub sam: &'s GeneralSAM<T>,
    pub node_id: usize,
}

impl<'s> GeneralSAMState<'s, u8> {
    pub fn feed_bytes(self, seq: &'s str) -> Self {
        self.feed_ref(seq.as_bytes())
    }
}

impl GeneralSAMState<'_, char> {
    pub fn feed_chars(self, seq: &str) -> Self {
        self.feed(seq.chars())
    }
}

impl<T: Ord + Clone> GeneralSAMState<'_, T> {
    pub fn is_nil(&self) -> bool {
        self.node_id == SAM_NIL_NODE_ID
    }

    pub fn is_root(&self) -> bool {
        self.node_id == SAM_ROOT_NODE_ID
    }

    pub fn is_accepting(&self) -> bool {
        self.get_node()
            .map(|node| node.is_accepting())
            .unwrap_or(false)
    }

    pub fn has_trans(&self, key: &T) -> bool {
        self.get_node()
            .map(|node| node.trans.contains_key(key))
            .unwrap_or(false)
    }

    pub fn get_node(&self) -> Option<&GeneralSAMNode<T>> {
        self.sam.get_node(self.node_id)
    }

    pub fn goto_suffix_parent(&mut self) {
        if let Some(node) = self.get_node() {
            self.node_id = node.link;
        } else {
            self.node_id = SAM_NIL_NODE_ID;
        }
    }

    pub fn goto(&mut self, t: &T) {
        self.node_id =
            if let Some(next_node_id) = self.get_node().and_then(|node| node.trans.get(t)) {
                *next_node_id
            } else {
                SAM_NIL_NODE_ID
            }
    }

    pub fn feed<Seq: IntoIterator<Item = T>>(self, seq: Seq) -> Self {
        self.feed_iter(seq.into_iter())
    }

    pub fn feed_iter<Iter: Iterator<Item = T>>(mut self, iter: Iter) -> Self {
        for t in iter {
            if self.is_nil() {
                break;
            }
            self.goto(&t)
        }
        self
    }
}

impl<'s, T: Ord + Clone> GeneralSAMState<'s, T> {
    pub fn feed_ref<Seq: IntoIterator<Item = &'s T>>(self, seq: Seq) -> Self {
        self.feed_ref_iter(seq.into_iter())
    }

    pub fn feed_ref_iter<Iter: Iterator<Item = &'s T>>(mut self, iter: Iter) -> Self {
        for t in iter {
            if self.is_nil() {
                break;
            }
            self.goto(t)
        }
        self
    }
}

impl<'s, T: Ord + Clone> GeneralSAMState<'s, T> {
    fn wrap_travel_along_callback<
        TN: TrieNodeAlike<InnerType = T>,
        ExtraType,
        ErrorType,
        F: 's
            + FnMut(
                TravelEvent<(&GeneralSAMState<T>, &TN), ExtraType, TN::InnerType>,
            ) -> Result<ExtraType, ErrorType>,
    >(
        &'s self,
        mut callback: F,
    ) -> impl FnMut(
        TravelEvent<&TN, (GeneralSAMState<'s, T>, ExtraType), TN::InnerType>,
    ) -> Result<(GeneralSAMState<'s, T>, ExtraType), ErrorType> {
        move |event| match event {
            TravelEvent::PushRoot(trie_root) => {
                let res = callback(TravelEvent::PushRoot((self, trie_root)))?;
                Ok((self.clone(), res))
            }
            TravelEvent::Push(cur_tn, (cur_state, cur_extra), key) => {
                let mut next_state = cur_state.clone();
                next_state.goto(&key);
                let next_extra =
                    callback(TravelEvent::Push((&next_state, &cur_tn), cur_extra, key))?;
                Ok((next_state, next_extra))
            }
            TravelEvent::Pop(cur_tn, (cur_state, extra)) => {
                let res = callback(TravelEvent::Pop((&cur_state, cur_tn), extra))?;
                Ok((cur_state, res))
            }
        }
    }

    pub fn dfs_along<
        TN: TrieNodeAlike<InnerType = T> + Clone,
        ExtraType,
        ErrorType,
        F: FnMut(
            TravelEvent<(&GeneralSAMState<'_, T>, &TN), ErrorType, TN::InnerType>,
        ) -> Result<ErrorType, ExtraType>,
    >(
        &self,
        trie_node: TN,
        callback: F,
    ) -> Result<(), ExtraType> {
        trie_node.dfs_travel(self.wrap_travel_along_callback(callback))
    }

    pub fn bfs_along<
        TN: TrieNodeAlike<InnerType = T>,
        ExtraType,
        ErrorType,
        F: FnMut(
            TravelEvent<(&GeneralSAMState<'_, T>, &TN), ErrorType, TN::InnerType>,
        ) -> Result<ErrorType, ExtraType>,
    >(
        &self,
        trie_node: TN,
        callback: F,
    ) -> Result<(), ExtraType> {
        trie_node.bfs_travel(self.wrap_travel_along_callback(callback))
    }
}
