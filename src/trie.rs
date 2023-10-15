use std::collections::{btree_map, BTreeMap};

use crate::trie_alike::TrieNodeAlike;

pub const TRIE_NIL_NODE_ID: usize = 0;
pub const TRIE_ROOT_NODE_ID: usize = 1;

#[derive(Debug, Clone)]
pub struct TrieNode<T: Ord + Clone> {
    trans: BTreeMap<T, usize>,
    parent: usize,
    pub accept: bool,
}

#[derive(Debug, Clone)]
pub struct Trie<T: Ord + Clone> {
    node_pool: Vec<TrieNode<T>>,
}

#[derive(Debug, Clone)]
pub struct TrieState<'s, T: Ord + Clone> {
    pub trie: &'s Trie<T>,
    pub node_id: usize,
}

impl<T: Ord + Clone> TrieNode<T> {
    fn new(parent: usize) -> Self {
        Self {
            trans: Default::default(),
            parent,
            accept: Default::default(),
        }
    }

    pub fn get_trans(&self) -> &BTreeMap<T, usize> {
        &self.trans
    }

    pub fn get_parent(&self) -> usize {
        self.parent
    }
}

impl<T: Ord + Clone> Default for Trie<T> {
    fn default() -> Self {
        Self {
            node_pool: vec![TrieNode::new(TRIE_NIL_NODE_ID), TrieNode::new(TRIE_NIL_NODE_ID)],
        }
    }
}

impl<T: Ord + Clone> Trie<T> {
    pub fn num_of_nodes(&self) -> usize {
        self.node_pool.len()
    }

    pub fn get_state(&self, node_id: usize) -> TrieState<T> {
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

    pub fn get_node(&self, node_id: usize) -> Option<&TrieNode<T>> {
        self.node_pool.get(node_id)
    }

    pub fn get_root_node(&self) -> &TrieNode<T> {
        self.get_node(TRIE_ROOT_NODE_ID).unwrap()
    }

    pub fn get_root_state(&self) -> TrieState<T> {
        self.get_state(TRIE_ROOT_NODE_ID)
    }

    fn alloc_node(&mut self, parent: usize) -> usize {
        let node_id = self.node_pool.len();
        self.node_pool.push(TrieNode::new(parent));
        node_id
    }

    pub fn insert_ref_iter<'s, Iter: Iterator<Item = &'s T>>(&'s mut self, iter: Iter) -> usize {
        self.insert_iter(iter.cloned())
    }

    pub fn insert_iter<Iter: Iterator<Item = T>>(&mut self, iter: Iter) -> usize {
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

impl<'s, T: Ord + Clone> TrieState<'s, T> {
    pub fn is_nil(&self) -> bool {
        self.node_id == TRIE_NIL_NODE_ID
    }

    pub fn is_root(&self) -> bool {
        self.node_id == TRIE_ROOT_NODE_ID
    }

    pub fn get_node(&self) -> Option<&'s TrieNode<T>> {
        self.trie.get_node(self.node_id)
    }

    pub fn goto_parent(&mut self) {
        if let Some(node) = self.get_node() {
            self.node_id = node.parent;
        } else {
            self.node_id = TRIE_NIL_NODE_ID;
        }
    }

    pub fn goto(&mut self, t: &T) {
        if let Some(node) = self.get_node() {
            self.node_id = node.trans.get(t).copied().unwrap_or(TRIE_NIL_NODE_ID)
        } else {
            self.node_id = TRIE_NIL_NODE_ID;
        }
    }
}

#[derive(Debug)]
pub struct NextStateIter<'s, T: Ord + Clone> {
    state: TrieState<'s, T>,
    iter: btree_map::Iter<'s, T, usize>,
}

impl<'s, T: Ord + Clone> TrieNodeAlike for TrieState<'s, T> {
    type InnerType = T;
    type NextStateIter = NextStateIter<'s, T>;

    fn is_accepting(&self) -> bool {
        self.get_node().map(|x| x.accept).unwrap_or(false)
    }

    fn next_states(self) -> NextStateIter<'s, T> {
        let iter = self.get_node().unwrap().trans.iter();
        NextStateIter { state: self, iter }
    }
}

impl<'s, T: Ord + Clone> Iterator for NextStateIter<'s, T> {
    type Item = (T, TrieState<'s, T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(t, next_node_id)| (t.clone(), self.state.trie.get_state(*next_node_id)))
    }
}

impl<'s, T: Ord + Clone> NextStateIter<'s, T> {
    pub fn get_state(&self) -> &TrieState<T> {
        &self.state
    }
}
