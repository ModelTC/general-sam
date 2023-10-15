use std::collections::VecDeque;

pub enum TravelEvent<NodeType, KeyType> {
    Push(NodeType, Option<KeyType>),
    Pop(NodeType),
}

/// This trait provides the essential interfaces required by `GeneralSAM`
/// to construct a suffix automaton from structures that form a prefix tree.
pub trait TrieNodeAlike {
    type InnerType;
    type NextStateIter: Iterator<Item = (Self::InnerType, Self)>;
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Self::NextStateIter;

    fn bfs_travel<E, F: FnMut(TravelEvent<&Self, Self::InnerType>) -> Result<(), E>>(
        self,
        mut callback: F,
    ) -> Result<(), E>
    where
        Self: Sized,
    {
        let mut queue = VecDeque::new();
        callback(TravelEvent::Push(&self, None))?;
        queue.push_back(self);
        while let Some(state) = queue.pop_front() {
            callback(TravelEvent::Pop(&state))?;
            for (t, v) in state.next_states() {
                callback(TravelEvent::Push(&v, Some(t)))?;
                queue.push_back(v);
            }
        }
        Ok(())
    }

    fn dfs_travel<E, F: FnMut(TravelEvent<&Self, Self::InnerType>) -> Result<(), E>>(
        self,
        mut callback: F,
    ) -> Result<(), E>
    where
        Self: Clone,
    {
        let mut stack = Vec::new();

        callback(TravelEvent::Push(&self, None))?;
        stack.push((self.clone(), self.next_states()));

        while let Some((ref cur, ref mut iter)) = stack.last_mut() {
            if let Some((key, next_state)) = iter.next() {
                callback(TravelEvent::Push(&next_state, Some(key)))?;
                stack.push((next_state.clone(), next_state.next_states()));
            } else {
                callback(TravelEvent::Pop(cur))?;
                stack.pop();
            }
        }
        Ok(())
    }
}

// This struct implements `TrieNodeAlike` for any iterator.
//
// It can be used to construct a suffix automaton directly from a sequence.
pub struct IterAsChain<Iter: Iterator> {
    iter: Iter,
    val: Option<Iter::Item>,
}

impl<Iter: Iterator> From<Iter> for IterAsChain<Iter> {
    fn from(mut iter: Iter) -> Self {
        let val = iter.next();
        Self { iter, val }
    }
}

impl<Iter: Iterator> TrieNodeAlike for IterAsChain<Iter> {
    type InnerType = Iter::Item;
    type NextStateIter = IterAsChainDummyNextState<Iter>;

    fn is_accepting(&self) -> bool {
        self.val.is_none()
    }

    fn next_states(self) -> Self::NextStateIter {
        Self::NextStateIter { state: Some(self) }
    }
}

pub struct IterAsChainDummyNextState<Iter: Iterator> {
    state: Option<IterAsChain<Iter>>,
}

impl<Iter: Iterator> Iterator for IterAsChainDummyNextState<Iter> {
    type Item = (Iter::Item, IterAsChain<Iter>);

    fn next(&mut self) -> Option<Self::Item> {
        self.state.take().and_then(|mut chain| {
            if let Some(v) = chain.val {
                chain.val = chain.iter.next();
                Some((v, chain))
            } else {
                None
            }
        })
    }
}
