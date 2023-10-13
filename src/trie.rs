use std::collections::{btree_map, BTreeMap};

use crate::trie_alike::TrieNodeAlike;

pub const TRIE_NIL_NODE_ID: usize = 0;
pub const TRIE_ROOT_NODE_ID: usize = 1;

#[derive(Debug, Clone)]
pub struct Node<T: Ord + Clone> {
    trans: BTreeMap<T, usize>,
    parent: usize,
    pub accept: bool,
}

#[derive(Debug, Clone)]
pub struct Trie<T: Ord + Clone> {
    node_pool: Vec<Node<T>>,
}

#[derive(Debug, Clone)]
pub struct State<'s, T: Ord + Clone> {
    pub trie: &'s Trie<T>,
    pub node_id: usize,
}

impl<T: Ord + Clone> Node<T> {
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
            node_pool: vec![Node::new(TRIE_NIL_NODE_ID), Node::new(TRIE_NIL_NODE_ID)],
        }
    }
}

impl<T: Ord + Clone> Trie<T> {
    pub fn get_state(&self, node_id: usize) -> State<T> {
        if node_id >= self.node_pool.len() {
            return State {
                trie: self,
                node_id: TRIE_NIL_NODE_ID,
            };
        }
        State {
            trie: self,
            node_id,
        }
    }

    pub fn get_node(&self, node_id: usize) -> Option<&Node<T>> {
        self.node_pool.get(node_id)
    }

    pub fn get_root_node(&self) -> &Node<T> {
        self.get_node(TRIE_ROOT_NODE_ID).unwrap()
    }

    pub fn get_root_state(&self) -> State<T> {
        self.get_state(TRIE_ROOT_NODE_ID)
    }

    fn alloc_node(&mut self, parent: usize) -> usize {
        let node_id = self.node_pool.len();
        self.node_pool.push(Node::new(parent));
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

    pub fn get_bfs_order(&self) -> Vec<usize> {
        let mut res = Vec::new();
        let mut head = 0;
        res.push(TRIE_ROOT_NODE_ID);
        while head < res.len() {
            let cur_id = res[head];
            head += 1;
            self.node_pool[cur_id].trans.values().for_each(|v| {
                res.push(*v);
            });
        }
        res
    }
}

impl<'s, T: Ord + Clone> State<'s, T> {
    pub fn is_nil(&self) -> bool {
        self.node_id == TRIE_NIL_NODE_ID
    }

    pub fn is_root(&self) -> bool {
        self.node_id == TRIE_ROOT_NODE_ID
    }

    pub fn get_node(&self) -> Option<&'s Node<T>> {
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
    state: State<'s, T>,
    iter: btree_map::Iter<'s, T, usize>,
}

impl<'s, T: Ord + Clone> TrieNodeAlike for State<'s, T> {
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
    type Item = (T, State<'s, T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(t, next_node_id)| (t.clone(), self.state.trie.get_state(*next_node_id)))
    }
}

impl<'s, T: Ord + Clone> NextStateIter<'s, T> {
    pub fn get_state(&self) -> &State<T> {
        &self.state
    }
}
