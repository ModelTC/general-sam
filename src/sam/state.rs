//! States of a general suffix automaton.

use std::{borrow::Borrow, marker::PhantomData};

use crate::{TravelEvent, TrieNodeAlike};

use super::{GeneralSam, GeneralSamNode, TransitionTable, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID};

#[derive(Debug)]
pub struct GeneralSamState<TransTable: TransitionTable, SamRef: Borrow<GeneralSam<TransTable>>> {
    pub sam: SamRef,
    pub node_id: usize,
    phantom: PhantomData<TransTable>,
}

impl<TransTable: TransitionTable, SamRef: Borrow<GeneralSam<TransTable>> + Clone> Clone
    for GeneralSamState<TransTable, SamRef>
{
    fn clone(&self) -> Self {
        Self {
            sam: self.sam.clone(),
            node_id: self.node_id,
            phantom: PhantomData,
        }
    }
}

impl<TransTable: TransitionTable<KeyType = u8>, SamRef: Borrow<GeneralSam<TransTable>>>
    GeneralSamState<TransTable, SamRef>
{
    pub fn feed_bytes<S: AsRef<[u8]>>(&mut self, seq: S) -> &mut Self {
        self.feed_ref(seq.as_ref())
    }
}

impl<TransTable: TransitionTable<KeyType = char>, SamRef: Borrow<GeneralSam<TransTable>>>
    GeneralSamState<TransTable, SamRef>
{
    pub fn feed_chars<S: AsRef<str>>(&mut self, seq: S) -> &mut Self {
        self.feed(seq.as_ref().chars())
    }
}

impl<TransTable: TransitionTable, SamRef: Borrow<GeneralSam<TransTable>>>
    GeneralSamState<TransTable, SamRef>
{
    pub fn new(sam: SamRef, node_id: usize) -> Self {
        Self {
            sam,
            node_id,
            phantom: PhantomData,
        }
    }

    pub fn inner_as_ref(&self) -> GeneralSamState<TransTable, &GeneralSam<TransTable>> {
        GeneralSamState {
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

    pub fn get_sam_ref(&self) -> &GeneralSam<TransTable> {
        self.sam.borrow()
    }

    pub fn get_node(&self) -> Option<&GeneralSamNode<TransTable>> {
        self.sam.borrow().get_node(self.node_id)
    }

    pub fn goto_suffix_parent(&mut self) -> &mut Self {
        if let Some(node) = self.get_node() {
            self.node_id = node.link;
        } else {
            self.node_id = SAM_NIL_NODE_ID;
        }
        self
    }

    pub fn goto<K: Borrow<TransTable::KeyType>>(&mut self, t: &K) -> &mut Self {
        self.node_id = if let Some(next_node_id) =
            self.get_node().and_then(|node| node.trans.get(t.borrow()))
        {
            *next_node_id
        } else {
            SAM_NIL_NODE_ID
        };
        self
    }

    pub fn feed<Seq: IntoIterator<Item = TransTable::KeyType>>(&mut self, seq: Seq) -> &mut Self {
        for t in seq {
            if self.is_nil() {
                break;
            }
            self.goto(&t);
        }
        self
    }

    pub fn feed_ref<'s, Seq: IntoIterator<Item = &'s TransTable::KeyType>>(
        &mut self,
        seq: Seq,
    ) -> &mut Self
    where
        <TransTable as TransitionTable>::KeyType: 's,
    {
        for t in seq {
            if self.is_nil() {
                break;
            }
            self.goto(t);
        }
        self
    }
}

impl<TransTable: TransitionTable, SamRef: Borrow<GeneralSam<TransTable>> + Clone>
    GeneralSamState<TransTable, SamRef>
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
