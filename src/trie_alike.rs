//! A trait for constructing `GeneralSAM` from structures that form a trie,
//! and some utilities to construct `GeneralSAM` from iterators.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum TravelEvent<'s, NodeType, ExtraType, KeyType> {
    PushRoot(NodeType),
    Push(NodeType, &'s ExtraType, KeyType),
    Pop(NodeType, ExtraType),
}

/// This trait provides the essential interfaces required by `GeneralSAM`
/// to construct a suffix automaton from structures that form a trie (prefix tree).
pub trait TrieNodeAlike {
    type InnerType;
    type NextStateIter: Iterator<Item = (Self::InnerType, Self)>;
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Self::NextStateIter;

    fn bfs_travel<
        ErrorType,
        ExtraType,
        F: FnMut(TravelEvent<&Self, ExtraType, Self::InnerType>) -> Result<ExtraType, ErrorType>,
    >(
        self,
        mut callback: F,
    ) -> Result<(), ErrorType>
    where
        Self: Sized,
    {
        let mut queue = VecDeque::new();

        let extra = callback(TravelEvent::PushRoot(&self))?;
        queue.push_back((self, extra));

        while let Some((state, cur_extra)) = queue.pop_front() {
            let cur_extra = callback(TravelEvent::Pop(&state, cur_extra))?;

            for (t, v) in state.next_states() {
                let next_extra = callback(TravelEvent::Push(&v, &cur_extra, t))?;
                queue.push_back((v, next_extra));
            }
        }
        Ok(())
    }

    fn dfs_travel<
        ErrorType,
        ExtraType,
        F: FnMut(TravelEvent<&Self, ExtraType, Self::InnerType>) -> Result<ExtraType, ErrorType>,
    >(
        self,
        mut callback: F,
    ) -> Result<(), ErrorType>
    where
        Self: Clone,
    {
        let mut stack = Vec::new();

        let extra = callback(TravelEvent::PushRoot(&self))?;
        stack.push((self.clone(), self.next_states(), extra));

        while !stack.is_empty() {
            if let Some((_, iter, extra)) = stack.last_mut() {
                if let Some((key, next_state)) = iter.next() {
                    let new_extra = callback(TravelEvent::Push(&next_state, extra, key))?;
                    stack.push((next_state.clone(), next_state.next_states(), new_extra));
                    continue;
                }
            }
            if let Some((cur, _, extra)) = stack.pop() {
                callback(TravelEvent::Pop(&cur, extra))?;
            }
        }
        Ok(())
    }
}

/// This struct implements `TrieNodeAlike` for any iterator.
///
/// It can be used to construct a suffix automaton directly from a sequence.
#[derive(Clone, Debug)]
pub struct IterAsChain<Iter: Iterator> {
    pub iter: Iter,
    pub val: Option<Iter::Item>,
}

pub struct IterAsChainNextStateIter<Iter: Iterator> {
    pub state: Option<IterAsChain<Iter>>,
}

impl<Iter: Iterator> From<Iter> for IterAsChain<Iter> {
    fn from(mut iter: Iter) -> Self {
        let val = iter.next();
        Self { iter, val }
    }
}

impl<Iter: Iterator> TrieNodeAlike for IterAsChain<Iter> {
    type InnerType = Iter::Item;
    type NextStateIter = IterAsChainNextStateIter<Iter>;

    fn is_accepting(&self) -> bool {
        self.val.is_none()
    }

    fn next_states(self) -> Self::NextStateIter {
        Self::NextStateIter { state: Some(self) }
    }
}

impl<Iter: Iterator> Iterator for IterAsChainNextStateIter<Iter> {
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
