use super::{GeneralSAM, GeneralSAMNode, SAM_NIL_NODE_ID, SAM_ROOT_NODE_ID};

#[derive(Debug, Clone)]
pub struct GeneralSAMState<'s, T: Ord + Clone> {
    pub sam: &'s GeneralSAM<T>,
    pub node_id: usize,
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

    pub fn get_node(&self) -> Option<&'_ GeneralSAMNode<T>> {
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
