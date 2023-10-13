pub trait TrieNodeAlike {
    type InnerType;
    type NextStateIter: Iterator<Item = (Self::InnerType, Self)>;
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Self::NextStateIter;
}

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
