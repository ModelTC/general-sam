//! States of a general suffix automaton.

use std::{borrow::Borrow, marker::PhantomData};

use crate::{TravelEvent, TrieNodeAlike};

use super::{GeneralSAM, GeneralSAMNode, TransitionTable, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID};

#[derive(Debug)]
pub struct GeneralSAMState<TransTable: TransitionTable, SAMRef: Borrow<GeneralSAM<TransTable>>> {
    pub sam: SAMRef,
    pub node_id: usize,
    phantom: PhantomData<TransTable>,
}

impl<TransTable: TransitionTable, SAMRef: Borrow<GeneralSAM<TransTable>> + Clone> Clone
    for GeneralSAMState<TransTable, SAMRef>
{
    fn clone(&self) -> Self {
        Self {
            sam: self.sam.clone(),
            node_id: self.node_id,
            phantom: PhantomData,
        }
    }
}

impl<TransTable: TransitionTable<KeyType = u8>, SAMRef: Borrow<GeneralSAM<TransTable>>>
    GeneralSAMState<TransTable, SAMRef>
{
    pub fn feed_bytes(self, seq: &str) -> Self {
        self.feed_ref(seq.as_bytes())
    }
}

impl<TransTable: TransitionTable<KeyType = char>, SAMRef: Borrow<GeneralSAM<TransTable>>>
    GeneralSAMState<TransTable, SAMRef>
{
    pub fn feed_chars(self, seq: &str) -> Self {
        self.feed(seq.chars())
    }
}

impl<TransTable: TransitionTable, SAMRef: Borrow<GeneralSAM<TransTable>>>
    GeneralSAMState<TransTable, SAMRef>
{
    pub fn new(sam: SAMRef, node_id: usize) -> Self {
        Self {
            sam,
            node_id,
            phantom: PhantomData,
        }
    }

    pub fn inner_as_ref(&self) -> GeneralSAMState<TransTable, &GeneralSAM<TransTable>> {
        GeneralSAMState {
            sam: self.sam.borrow(),
            node_id: self.node_id,
            phantom: PhantomData,
        }
    }

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

    pub fn get_sam_ref(&self) -> &GeneralSAM<TransTable> {
        self.sam.borrow()
    }

    pub fn get_node(&self) -> Option<&GeneralSAMNode<TransTable>> {
        self.sam.borrow().get_node(self.node_id)
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

    pub fn feed_ref<'s, Seq: IntoIterator<Item = &'s TransTable::KeyType>>(self, seq: Seq) -> Self
    where
        <TransTable as TransitionTable>::KeyType: 's,
    {
        self.feed_ref_iter(seq.into_iter())
    }

    pub fn feed_ref_iter<'s, Iter: Iterator<Item = &'s TransTable::KeyType>>(
        mut self,
        iter: Iter,
    ) -> Self
    where
        <TransTable as TransitionTable>::KeyType: 's,
    {
        for t in iter {
            if self.is_nil() {
                break;
            }
            self.goto(t)
        }
        self
    }
}

impl<TransTable: TransitionTable, SAMRef: Borrow<GeneralSAM<TransTable>> + Clone>
    GeneralSAMState<TransTable, SAMRef>
{
    pub fn get_non_nil_trans(&self, key: &TransTable::KeyType) -> Option<Self> {
        self.get_node()
            .and_then(|node| node.trans.get(key))
            .map(|x| Self {
                sam: self.sam.clone(),
                node_id: *x,
                phantom: PhantomData,
            })
    }

    fn wrap_travel_along_callback<
        's,
        TN: TrieNodeAlike<InnerType = TransTable::KeyType>,
        ExtraType,
        ErrorType,
        F: 's
            + FnMut(
                TravelEvent<(&Self, &TN), ExtraType, TN::InnerType>,
            ) -> Result<ExtraType, ErrorType>,
    >(
        &'s self,
        mut callback: F,
    ) -> impl FnMut(
        TravelEvent<&TN, (Self, ExtraType), TN::InnerType>,
    ) -> Result<(Self, ExtraType), ErrorType>
           + 's {
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
        F: FnMut(TravelEvent<(&Self, &TN), ErrorType, TN::InnerType>) -> Result<ErrorType, ExtraType>,
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
        F: FnMut(TravelEvent<(&Self, &TN), ErrorType, TN::InnerType>) -> Result<ErrorType, ExtraType>,
    >(
        &self,
        trie_node: TN,
        callback: F,
    ) -> Result<(), ExtraType> {
        trie_node.bfs_travel(self.wrap_travel_along_callback(callback))
    }
}
