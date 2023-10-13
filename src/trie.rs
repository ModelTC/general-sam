use std::collections::{btree_map, BTreeMap};

use crate::trie_alike::TrieNodeAlike;

#[derive(Debug)]
pub struct Node<T>
where
    T: Ord + Copy,
{
    trans: BTreeMap<T, usize>,
    pub accept: bool,
}

#[derive(Debug)]
pub struct Trie<T>
where
    T: Ord + Copy,
{
    node_pool: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct State<'s, T>
where
    T: Ord + Copy,
{
    pub trie: &'s Trie<T>,
    node_id: usize,
}

#[derive(Debug)]
pub struct NextStateIter<'s, T>
where
    T: Ord + Copy,
{
    state: State<'s, T>,
    iter: btree_map::Iter<'s, T, usize>,
}

impl<T: Ord + Copy> Trie<T> {
    pub fn get_state(&self, node_id: usize) -> State<T> {
        State {
            trie: self,
            node_id,
        }
    }
}

impl<'s, T: Ord + Copy> State<'s, T> {
    pub fn get_node(&self) -> &'s Node<T> {
        &self.trie.node_pool[self.node_id]
    }
}

impl<'s, T: Ord + Copy> TrieNodeAlike for State<'s, T> {
    type InnerType = T;
    type NextStateIter = NextStateIter<'s, T>;

    fn is_accepting(&self) -> bool {
        self.get_node().accept
    }

    fn next_states(self) -> NextStateIter<'s, T> {
        let iter = self.get_node().trans.iter();
        NextStateIter { state: self, iter }
    }
}

impl<'s, T: Ord + Copy> Iterator for NextStateIter<'s, T> {
    type Item = (T, State<'s, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((t, next_node_id)) = self.iter.next() {
            Some((*t, self.state.trie.get_state(*next_node_id)))
        } else {
            None
        }
    }
}
