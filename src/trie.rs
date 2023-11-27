//! Trie, supporting `TrieNodeAlike`.

use std::ops::Deref;

use crate::{ConstructiveTransitionTable, GeneralSAMNodeID, TransitionTable, TrieNodeAlike};

pub type TrieNodeID = GeneralSAMNodeID;
pub const TRIE_NIL_NODE_ID: TrieNodeID = 0;
pub const TRIE_ROOT_NODE_ID: TrieNodeID = 1;

#[derive(Clone, Debug)]
pub struct TrieNode<TransTable: TransitionTable> {
    trans: TransTable,
    parent: TrieNodeID,
    pub accept: bool,
}

#[derive(Clone, Debug)]
pub struct Trie<TransTable: TransitionTable> {
    node_pool: Vec<TrieNode<TransTable>>,
}

#[derive(Debug)]
pub struct TrieState<TransTable: TransitionTable, TrieRef: Deref<Target = Trie<TransTable>>> {
    pub trie: TrieRef,
    pub node_id: TrieNodeID,
}

impl<TransTable: TransitionTable, TrieRef: Deref<Target = Trie<TransTable>> + Clone> Clone
    for TrieState<TransTable, TrieRef>
{
    fn clone(&self) -> Self {
        Self {
            trie: self.trie.clone(),
            node_id: self.node_id,
        }
    }
}

impl<TransTable: ConstructiveTransitionTable> TrieNode<TransTable> {
    fn new(parent: TrieNodeID) -> Self {
        Self {
            trans: Default::default(),
            parent,
            accept: Default::default(),
        }
    }
}

impl<TransTable: TransitionTable> TrieNode<TransTable> {
    pub fn get_trans(&self) -> &TransTable {
        &self.trans
    }

    pub fn get_parent(&self) -> TrieNodeID {
        self.parent
    }

    fn alter_trans_table<NewTableType: TransitionTable<KeyType = TransTable::KeyType>>(
        &self,
    ) -> TrieNode<NewTableType> {
        TrieNode {
            trans: NewTableType::from_kv_iter(self.trans.iter()),
            parent: self.parent,
            accept: self.accept,
        }
    }
}

impl<TransTable: ConstructiveTransitionTable> Default for Trie<TransTable> {
    fn default() -> Self {
        Self {
            node_pool: vec![
                TrieNode::new(TRIE_NIL_NODE_ID),
                TrieNode::new(TRIE_NIL_NODE_ID),
            ],
        }
    }
}

impl<TransTable: TransitionTable> Trie<TransTable> {
    pub fn num_of_nodes(&self) -> usize {
        self.node_pool.len()
    }

    pub fn get_state(&self, node_id: TrieNodeID) -> TrieState<TransTable, &Trie<TransTable>> {
        if node_id >= self.node_pool.len() {
            return TrieState {
                trie: self,
                node_id: TRIE_NIL_NODE_ID,
            };
        }
        TrieState {
            trie: self,
            node_id,
        }
    }

    pub fn get_node(&self, node_id: TrieNodeID) -> Option<&TrieNode<TransTable>> {
        self.node_pool.get(node_id)
    }

    pub fn get_root_node(&self) -> &TrieNode<TransTable> {
        self.get_node(TRIE_ROOT_NODE_ID).unwrap()
    }

    pub fn get_root_state(&self) -> TrieState<TransTable, &Trie<TransTable>> {
        self.get_state(TRIE_ROOT_NODE_ID)
    }

    pub fn alter_trans_table<NewTableType: TransitionTable<KeyType = TransTable::KeyType>>(
        &self,
    ) -> Trie<NewTableType> {
        Trie {
            node_pool: self
                .node_pool
                .iter()
                .map(|x| x.alter_trans_table())
                .collect(),
        }
    }
}

impl<TransTable: ConstructiveTransitionTable> Trie<TransTable> {
    fn alloc_node(&mut self, parent: TrieNodeID) -> TrieNodeID {
        let node_id = self.node_pool.len();
        self.node_pool.push(TrieNode::new(parent));
        node_id
    }

    pub fn insert_ref_iter<'s, Iter: Iterator<Item = &'s TransTable::KeyType>>(
        &'s mut self,
        iter: Iter,
    ) -> TrieNodeID {
        self.insert_iter(iter.cloned())
    }

    pub fn insert_iter<Iter: Iterator<Item = TransTable::KeyType>>(
        &mut self,
        iter: Iter,
    ) -> TrieNodeID {
        let mut current = TRIE_ROOT_NODE_ID;
        iter.for_each(|t| {
            current = match self.node_pool[current].trans.get(&t) {
                Some(v) => *v,
                None => {
                    let new_node_id = self.alloc_node(current);
                    self.node_pool[current].trans.insert(t, new_node_id);
                    new_node_id
                }
            };
        });
        self.node_pool[current].accept = true;
        current
    }
}

impl<TransTable: TransitionTable, TrieRef: Deref<Target = Trie<TransTable>>>
    TrieState<TransTable, TrieRef>
{
    pub fn inner_as_ref(&self) -> TrieState<TransTable, &Trie<TransTable>> {
        TrieState {
            trie: &self.trie,
            node_id: self.node_id,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.node_id == TRIE_NIL_NODE_ID
    }

    pub fn is_root(&self) -> bool {
        self.node_id == TRIE_ROOT_NODE_ID
    }

    pub fn get_node(&self) -> Option<&TrieNode<TransTable>> {
        self.trie.get_node(self.node_id)
    }

    pub fn goto_parent(&mut self) {
        if let Some(node) = self.get_node() {
            self.node_id = node.parent;
        } else {
            self.node_id = TRIE_NIL_NODE_ID;
        }
    }

    pub fn goto(&mut self, t: &TransTable::KeyType) {
        if let Some(node) = self.get_node() {
            self.node_id = node.trans.get(t).copied().unwrap_or(TRIE_NIL_NODE_ID)
        } else {
            self.node_id = TRIE_NIL_NODE_ID;
        }
    }

    pub fn feed_iter<Iter: Iterator<Item = TransTable::KeyType>>(&mut self, iter: Iter) {
        iter.for_each(|x| self.goto(&x));
    }

    pub fn feed_ref_iter<'s, Iter: Iterator<Item = &'s TransTable::KeyType>>(
        &'s mut self,
        iter: Iter,
    ) {
        iter.for_each(|x| self.goto(x));
    }
}

#[derive(Clone, Debug)]
pub struct NextTrieStateIter<'s, TransTable: TransitionTable> {
    trie: &'s Trie<TransTable>,
    iter: TransTable::IterType<'s>,
}

impl<'s, TransTable: TransitionTable> TrieNodeAlike
    for TrieState<TransTable, &'s Trie<TransTable>>
{
    type InnerType = TransTable::KeyType;
    type NextStateIter = NextTrieStateIter<'s, TransTable>;

    fn is_accepting(&self) -> bool {
        self.get_node().map(|x| x.accept).unwrap_or(false)
    }

    fn next_states(self) -> Self::NextStateIter {
        let iter = self.trie.get_node(self.node_id).unwrap().trans.iter();
        NextTrieStateIter {
            trie: self.trie,
            iter,
        }
    }
}

impl<'s, TransTable: TransitionTable> Iterator for NextTrieStateIter<'s, TransTable> {
    type Item = (
        TransTable::KeyType,
        TrieState<TransTable, &'s Trie<TransTable>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(t, next_node_id)| (t.clone(), self.trie.get_state(*next_node_id)))
    }
}
