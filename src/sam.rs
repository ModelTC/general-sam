use std::collections::{BTreeMap, VecDeque};

use crate::trie_alike::{IterAsChain, TrieNodeAlike};

pub const SAM_NIL_NODE_ID: usize = 0;
pub const SAM_ROOT_NODE_ID: usize = 1;

#[derive(Debug, Clone)]
pub struct Node<T: Ord + Clone> {
    trans: BTreeMap<T, usize>,
    accept: bool,
    len: usize,
    link: usize,
}

#[derive(Debug, Clone)]
pub struct GeneralSAM<T: Ord + Clone> {
    node_pool: Vec<Node<T>>,
    topo_order: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct State<'s, T: Ord + Clone> {
    pub sam: &'s GeneralSAM<T>,
    pub node_id: usize,
}

impl<T: Ord + Clone> Node<T> {
    fn new(accept: bool, len: usize, link: usize) -> Self {
        Self {
            trans: BTreeMap::new(),
            accept,
            len,
            link,
        }
    }

    pub fn is_accepting(&self) -> bool {
        self.accept
    }

    pub fn max_suffix_len(&self) -> usize {
        self.len
    }
}

impl GeneralSAM<u8> {
    pub fn construct_from_bytes<S: AsRef<[u8]>>(s: S) -> Self {
        let iter = IterAsChain::from(s.as_ref().iter().copied());
        Self::construct_from_trie(iter)
    }
}

impl GeneralSAM<u32> {
    pub fn construct_from_utf32<S: AsRef<[u32]>>(s: S) -> Self {
        let iter = IterAsChain::from(s.as_ref().iter().copied());
        Self::construct_from_trie(iter)
    }
}

impl GeneralSAM<char> {
    pub fn construct_from_chars<S: Iterator<Item = char>>(s: S) -> Self {
        let iter = IterAsChain::from(s);
        Self::construct_from_trie(iter)
    }
}

impl<T: Ord + Clone> Default for GeneralSAM<T> {
    fn default() -> Self {
        Self {
            node_pool: vec![
                Node::new(false, 0, SAM_NIL_NODE_ID),
                Node::new(true, 0, SAM_NIL_NODE_ID),
            ],
            topo_order: Default::default(),
        }
    }
}

impl<T: Ord + Clone> GeneralSAM<T> {
    pub fn get_root_state(&self) -> State<T> {
        self.get_state(SAM_ROOT_NODE_ID)
    }

    pub fn get_state(&self, node_id: usize) -> State<T> {
        if node_id < self.node_pool.len() {
            State { sam: self, node_id }
        } else {
            State {
                sam: self,
                node_id: SAM_NIL_NODE_ID,
            }
        }
    }

    pub fn get_topo_order(&self) -> TopoOrderStateIter<T> {
        TopoOrderStateIter {
            sam: self,
            head: 0,
            rear: self.topo_order.len(),
        }
    }

    pub fn construct_from_trie<TN: TrieNodeAlike>(node: TN) -> Self
    where
        TN::InnerType: Into<T>,
    {
        let mut sam = Self::default();

        let accept_empty_string = node.is_accepting();

        sam.build_with_trie(node);
        sam.topo_sort();
        sam.update_accepting();

        sam.node_pool[SAM_ROOT_NODE_ID].accept = accept_empty_string;

        sam
    }

    fn build_with_trie<TN: TrieNodeAlike>(&mut self, node: TN)
    where
        TN::InnerType: Into<T>,
    {
        let mut queue = VecDeque::new();
        queue.push_back((SAM_ROOT_NODE_ID, node));
        while let Some((last_node_id, tn)) = queue.pop_front() {
            tn.next_states().for_each(|(key, next_tn)| {
                let new_node_id = self.insert_node_trans(last_node_id, key, next_tn.is_accepting());
                queue.push_back((new_node_id, next_tn));
            });
        }
    }

    fn topo_sort(&mut self) {
        let mut in_degree: Vec<usize> = Vec::new();
        in_degree.resize(self.node_pool.len(), 0);
        self.node_pool.iter().for_each(|node| {
            node.trans.values().for_each(|v| {
                in_degree[*v] += 1;
            });
        });
        assert!(in_degree[SAM_ROOT_NODE_ID] == 0);

        self.topo_order.reserve(self.node_pool.len());

        self.topo_order.push(SAM_ROOT_NODE_ID);
        let mut head = 0;
        while head < self.topo_order.len() {
            let u_id = self.topo_order[head];
            head += 1;
            self.node_pool[u_id].trans.values().for_each(|v_id| {
                in_degree[*v_id] -= 1;
                if in_degree[*v_id] == 0 {
                    self.topo_order.push(*v_id);
                }
            });
        }
    }

    fn update_accepting(&mut self) {
        self.topo_order.iter().rev().for_each(|node_id| {
            let link_id = self.node_pool[*node_id].link;
            self.node_pool[link_id].accept |= self.node_pool[*node_id].accept;
        });
        self.node_pool[SAM_NIL_NODE_ID].accept = false;
    }

    fn alloc_node(&mut self, node: Node<T>) -> usize {
        let id = self.node_pool.len();
        self.node_pool.push(node);
        id
    }

    fn insert_node_trans<Key: Into<T>>(
        &mut self,
        last_node_id: usize,
        key: Key,
        accept: bool,
    ) -> usize {
        let key: T = key.into();

        let new_node_id = {
            let last_node = &self.node_pool[last_node_id];
            self.alloc_node(Node::new(accept, last_node.len + 1, SAM_NIL_NODE_ID))
        };

        let mut p_node_id = last_node_id;
        while p_node_id != SAM_NIL_NODE_ID {
            let p_node = &mut self.node_pool[p_node_id];
            if p_node.trans.contains_key(&key) {
                break;
            }
            p_node.trans.insert(key.clone(), new_node_id);
            p_node_id = p_node.link;
        }

        if p_node_id == SAM_NIL_NODE_ID {
            self.node_pool[new_node_id].link = SAM_ROOT_NODE_ID;
            return new_node_id;
        }

        let q_node_id = *self.node_pool[p_node_id].trans.get(&key).unwrap();
        let q_node = &self.node_pool[q_node_id];
        if q_node.len == self.node_pool[p_node_id].len + 1 {
            self.node_pool[new_node_id].link = q_node_id;
            return new_node_id;
        }

        let clone_node_id = self.alloc_node(q_node.clone());
        self.node_pool[clone_node_id].len = self.node_pool[p_node_id].len + 1;
        while p_node_id != SAM_NIL_NODE_ID {
            let p_node = &mut self.node_pool[p_node_id];
            if let Some(t_node_id) = p_node.trans.get_mut(&key) {
                if *t_node_id == q_node_id {
                    *t_node_id = clone_node_id;
                    p_node_id = p_node.link;
                    continue;
                }
            }
            break;
        }

        self.node_pool[new_node_id].link = clone_node_id;
        self.node_pool[q_node_id].link = clone_node_id;

        new_node_id
    }
}

impl<'s, T: Ord + Clone> State<'s, T> {
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

    pub fn get_node(&self) -> Option<&'_ Node<T>> {
        self.sam.node_pool.get(self.node_id)
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

impl<'s> State<'s, u8> {
    pub fn feed_bytes(self, seq: &'s str) -> Self {
        self.feed_ref(seq.as_bytes())
    }
}

impl<'s> State<'s, char> {
    pub fn feed_chars(self, seq: &'s str) -> Self {
        self.feed(seq.chars())
    }
}

pub struct TopoOrderStateIter<'s, T: Ord + Clone> {
    sam: &'s GeneralSAM<T>,
    head: usize,
    rear: usize,
}

impl<'s, T: Ord + Clone> Iterator for TopoOrderStateIter<'s, T> {
    type Item = State<'s, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head >= self.sam.topo_order.len() {
            return None;
        }
        let res = State {
            sam: self.sam,
            node_id: self.sam.topo_order[self.head],
        };
        self.head += 1;
        Some(res)
    }
}
impl<'s, T: Ord + Clone> DoubleEndedIterator for TopoOrderStateIter<'s, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.rear == 0 {
            return None;
        }
        self.rear -= 1;
        let res = State {
            sam: self.sam,
            node_id: self.sam.topo_order[self.rear],
        };
        Some(res)
    }
}
