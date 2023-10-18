use std::collections::VecDeque;

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

impl<'s> GeneralSAMState<'s, char> {
    pub fn feed_chars(self, seq: &'s str) -> Self {
        self.feed(seq.chars())
    }
}

impl<'s, T: Ord + Clone> GeneralSAMState<'s, T> {
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

    pub fn bfs_along<
        TN: TrieNodeAlike<InnerType = T> + Sized,
        E,
        F: FnMut(TravelEvent<(GeneralSAMState<'_, T>, &TN), TN::InnerType>) -> Result<(), E>,
    >(
        &self,
        trie_node: TN,
        mut callback: F,
    ) -> Result<(), E> {
        let mut queue = VecDeque::new();
        let mut cur_node_id = self.node_id;

        trie_node.bfs_travel(|event| match event {
            TravelEvent::Push(tn, Some(key)) => {
                let next_node_id = self
                    .sam
                    .node_pool
                    .get(cur_node_id)
                    .and_then(|x| x.trans.get(&key).copied())
                    .unwrap_or(SAM_NIL_NODE_ID);
                callback(TravelEvent::Push(
                    (self.sam.get_state(next_node_id), tn),
                    Some(key),
                ))?;
                queue.push_back(next_node_id);
                Ok(())
            }
            TravelEvent::Push(tn, None) => {
                callback(TravelEvent::Push(
                    (self.sam.get_state(self.node_id), tn),
                    None,
                ))?;
                queue.push_back(self.node_id);
                Ok(())
            }
            TravelEvent::Pop(tn) => {
                cur_node_id = queue.pop_front().unwrap();
                callback(TravelEvent::Pop((self.sam.get_state(cur_node_id), tn)))?;
                Ok(())
            }
        })
    }

    pub fn dfs_along<
        TN: TrieNodeAlike<InnerType = T> + Clone,
        E,
        F: FnMut(TravelEvent<(GeneralSAMState<'_, T>, &TN), TN::InnerType>) -> Result<(), E>,
    >(
        &self,
        trie_node: TN,
        mut callback: F,
    ) -> Result<(), E> {
        let mut stack: Vec<usize> = Vec::new();

        trie_node.dfs_travel(|event| match event {
            TravelEvent::Push(tn, Some(key)) => {
                let next_node_id = self
                    .sam
                    .node_pool
                    .get(*stack.last().unwrap())
                    .and_then(|x| x.trans.get(&key).copied())
                    .unwrap_or(SAM_NIL_NODE_ID);
                callback(TravelEvent::Push(
                    (self.sam.get_state(next_node_id), tn),
                    Some(key),
                ))?;
                stack.push(next_node_id);
                Ok(())
            }
            TravelEvent::Push(tn, None) => {
                callback(TravelEvent::Push(
                    (self.sam.get_state(self.node_id), tn),
                    None,
                ))?;
                stack.push(self.node_id);
                Ok(())
            }
            TravelEvent::Pop(tn) => {
                let node_id = stack.pop().unwrap();
                callback(TravelEvent::Pop((self.sam.get_state(node_id), tn)))?;
                Ok(())
            }
        })
    }
}
