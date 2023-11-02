//! States of a general suffix automaton.

use crate::{TravelEvent, TrieNodeAlike};

use super::{GeneralSAM, GeneralSAMNode, TransitionTable, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID};

#[derive(Debug)]
pub struct GeneralSAMState<'s, TransTable: TransitionTable> {
    pub sam: &'s GeneralSAM<TransTable>,
    pub node_id: usize,
}

impl<'s, TransTable: TransitionTable> Clone for GeneralSAMState<'s, TransTable> {
    fn clone(&self) -> Self {
        Self {
            sam: self.sam,
            node_id: self.node_id,
        }
    }
}

impl<'s, TransTable: TransitionTable<KeyType = u8>> GeneralSAMState<'s, TransTable> {
    pub fn feed_bytes(self, seq: &'s str) -> Self {
        self.feed_ref(seq.as_bytes())
    }
}

impl<'s, TransTable: TransitionTable<KeyType = char>> GeneralSAMState<'s, TransTable> {
    pub fn feed_chars(self, seq: &str) -> Self {
        self.feed(seq.chars())
    }
}

impl<'s, TransTable: TransitionTable> GeneralSAMState<'s, TransTable> {
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

    pub fn get_non_nil_trans(&self, key: &TransTable::KeyType) -> Option<Self> {
        self.get_node()
            .and_then(|node| node.trans.get(key))
            .map(|x| self.sam.get_state(*x))
    }

    pub fn get_node(&self) -> Option<&GeneralSAMNode<TransTable>> {
        self.sam.get_node(self.node_id)
    }

    pub fn goto_suffix_parent(&mut self) {
        if let Some(node) = self.get_node() {
            self.node_id = node.link;
        } else {
            self.node_id = SAM_NIL_NODE_ID;
        }
    }

    pub fn goto(&mut self, t: &TransTable::KeyType) {
        self.node_id =
            if let Some(next_node_id) = self.get_node().and_then(|node| node.trans.get(t)) {
                *next_node_id
            } else {
                SAM_NIL_NODE_ID
            }
    }

    pub fn feed<Seq: IntoIterator<Item = TransTable::KeyType>>(self, seq: Seq) -> Self {
        self.feed_iter(seq.into_iter())
    }

    pub fn feed_iter<Iter: Iterator<Item = TransTable::KeyType>>(mut self, iter: Iter) -> Self {
        for t in iter {
            if self.is_nil() {
                break;
            }
            self.goto(&t)
        }
        self
    }
}

impl<'s, TransTable: TransitionTable> GeneralSAMState<'s, TransTable> {
    pub fn feed_ref<Seq: IntoIterator<Item = &'s TransTable::KeyType>>(self, seq: Seq) -> Self {
        self.feed_ref_iter(seq.into_iter())
    }

    pub fn feed_ref_iter<Iter: Iterator<Item = &'s TransTable::KeyType>>(
        mut self,
        iter: Iter,
    ) -> Self {
        for t in iter {
            if self.is_nil() {
                break;
            }
            self.goto(t)
        }
        self
    }

    fn wrap_travel_along_callback<
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        ExtraType,
        ErrorType,
        F: 's
            + FnMut(
                TravelEvent<(&GeneralSAMState<TransTable>, &TN), ExtraType, TN::InnerType>,
            ) -> Result<ExtraType, ErrorType>,
    >(
        &'s self,
        mut callback: F,
    ) -> impl FnMut(
        TravelEvent<&TN, (GeneralSAMState<'s, TransTable>, ExtraType), TN::InnerType>,
    ) -> Result<(GeneralSAMState<'s, TransTable>, ExtraType), ErrorType> {
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
        TN: TrieNodeAlike<InnerType = TransTable::KeyType> + Clone,
        ExtraType,
        ErrorType,
        F: FnMut(
            TravelEvent<(&GeneralSAMState<TransTable>, &TN), ErrorType, TN::InnerType>,
        ) -> Result<ErrorType, ExtraType>,
    >(
        &self,
        trie_node: TN,
        callback: F,
    ) -> Result<(), ExtraType> {
        trie_node.dfs_travel(self.wrap_travel_along_callback(callback))
    }

    pub fn bfs_along<
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        ExtraType,
        ErrorType,
        F: FnMut(
            TravelEvent<(&GeneralSAMState<TransTable>, &TN), ErrorType, TN::InnerType>,
        ) -> Result<ErrorType, ExtraType>,
    >(
        &self,
        trie_node: TN,
        callback: F,
    ) -> Result<(), ExtraType> {
        trie_node.bfs_travel(self.wrap_travel_along_callback(callback))
    }
}
