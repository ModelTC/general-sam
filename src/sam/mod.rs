mod state;
pub use state::GeneralSAMState;

use std::{
    collections::{BTreeMap, VecDeque},
    convert::Infallible,
};

use crate::trie_alike::{IterAsChain, TravelEvent, TrieNodeAlike};

pub type GeneralSAMNodeID = usize;
pub const SAM_NIL_NODE_ID: GeneralSAMNodeID = 0;
pub const SAM_ROOT_NODE_ID: GeneralSAMNodeID = 1;

#[derive(Debug, Clone)]
pub struct GeneralSAMNode<T: Ord + Clone> {
    trans: BTreeMap<T, GeneralSAMNodeID>,
    accept: bool,
    len: usize,
    link: GeneralSAMNodeID,
}

#[derive(Debug, Clone)]
pub struct GeneralSAM<T: Ord + Clone> {
    node_pool: Vec<GeneralSAMNode<T>>,
    topo_order: Vec<GeneralSAMNodeID>,
}

impl<T: Ord + Clone> GeneralSAMNode<T> {
    fn new(accept: bool, len: usize, link: GeneralSAMNodeID) -> Self {
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

    pub fn get_suffix_parent_id(&self) -> GeneralSAMNodeID {
        self.link
    }

    pub fn get_trans(&self) -> &BTreeMap<T, GeneralSAMNodeID> {
        &self.trans
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
                GeneralSAMNode::new(false, 0, SAM_NIL_NODE_ID),
                GeneralSAMNode::new(true, 0, SAM_NIL_NODE_ID),
            ],
            topo_order: Default::default(),
        }
    }
}

impl<T: Ord + Clone> GeneralSAM<T> {
    pub fn num_of_nodes(&self) -> usize {
        self.node_pool.len()
    }

    pub fn get_root_node(&self) -> &GeneralSAMNode<T> {
        self.get_node(SAM_ROOT_NODE_ID).unwrap()
    }

    pub fn get_node(&self, node_id: GeneralSAMNodeID) -> Option<&GeneralSAMNode<T>> {
        self.node_pool.get(node_id)
    }

    pub fn get_root_state(&self) -> GeneralSAMState<T> {
        self.get_state(SAM_ROOT_NODE_ID)
    }

    pub fn get_state(&self, node_id: GeneralSAMNodeID) -> GeneralSAMState<T> {
        if node_id < self.node_pool.len() {
            GeneralSAMState { sam: self, node_id }
        } else {
            GeneralSAMState {
                sam: self,
                node_id: SAM_NIL_NODE_ID,
            }
        }
    }

    pub fn get_topo_sorted_node_ids(&self) -> &Vec<GeneralSAMNodeID> {
        &self.topo_order
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
        let mut last_node_id = SAM_ROOT_NODE_ID;
        node.bfs_travel(|event| -> Result<(), Infallible> {
            match event {
                TravelEvent::Push(_, None) => {
                    queue.push_back(SAM_ROOT_NODE_ID);
                }
                TravelEvent::Pop(_) => {
                    last_node_id = queue.pop_front().unwrap();
                }
                TravelEvent::Push(tn, Some(key)) => {
                    let new_node_id = self.insert_node_trans(last_node_id, key, tn.is_accepting());
                    queue.push_back(new_node_id);
                }
            };
            Ok(())
        })
        .unwrap();
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

    fn alloc_node(&mut self, node: GeneralSAMNode<T>) -> GeneralSAMNodeID {
        let id = self.node_pool.len();
        self.node_pool.push(node);
        id
    }

    fn insert_node_trans<Key: Into<T>>(
        &mut self,
        last_node_id: GeneralSAMNodeID,
        key: Key,
        accept: bool,
    ) -> GeneralSAMNodeID {
        let key: T = key.into();

        let new_node_id = {
            let last_node = &self.node_pool[last_node_id];
            self.alloc_node(GeneralSAMNode::new(
                accept,
                last_node.len + 1,
                SAM_NIL_NODE_ID,
            ))
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
