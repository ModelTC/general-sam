use std::marker::PhantomData;

pub trait TrieNodeAlike<KeyType> {
    type NextStateIter: Iterator<Item = (KeyType, Self)>;
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Self::NextStateIter;
}

pub struct ByteChain<Iter: Iterator> {
    iter: Iter,
    val: Option<Iter::Item>,
}

impl<Iter: Iterator> From<Iter> for ByteChain<Iter> {
    fn from(mut iter: Iter) -> Self {
        let val = iter.next();
        Self { iter, val }
    }
}

impl<Iter: Iterator, KeyType: From<Iter::Item>> TrieNodeAlike<KeyType> for ByteChain<Iter> {
    type NextStateIter = ByteChainNextStateIter<KeyType, Iter>;

    fn is_accepting(&self) -> bool {
        self.val.is_none()
    }

    fn next_states(self) -> Self::NextStateIter {
        Self::NextStateIter {
            state: Some(self),
            phantom_key: PhantomData,
        }
    }
}

pub struct ByteChainNextStateIter<KeyType, Iter: Iterator> {
    state: Option<ByteChain<Iter>>,
    phantom_key: PhantomData<KeyType>,
}

impl<KeyType, Iter: Iterator> Iterator for ByteChainNextStateIter<KeyType, Iter>
where
    Iter::Item: Into<KeyType>,
{
    type Item = (KeyType, ByteChain<Iter>);

    fn next(&mut self) -> Option<Self::Item> {
        self.state.take().and_then(|mut chain| {
            if let Some(v) = chain.val {
                chain.val = chain.iter.next();
                Some((v.into(), chain))
            } else {
                None
            }
        })
    }
}

/*
// I was trying to make `Into` a duty of callers.
// It works, but rust-analyzer refused to hint for callers.
// I don't know why.

pub trait TrieNodeAlike {
    type InnerType;
    type NextStateIter: Iterator<Item = (Self::InnerType, Self)>;
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Self::NextStateIter;
}

pub struct ByteChain<Iter: Iterator> {
    iter: Iter,
    val: Option<Iter::Item>,
}

impl<Iter: Iterator> From<Iter> for ByteChain<Iter> {
    fn from(mut iter: Iter) -> Self {
        let val = iter.next();
        Self { iter, val }
    }
}

impl<Iter: Iterator> TrieNodeAlike for ByteChain<Iter> {
    type InnerType = Iter::Item;
    type NextStateIter = ByteChainNextStateIter<Iter>;

    fn is_accepting(&self) -> bool {
        self.val.is_none()
    }

    fn next_states(self) -> Self::NextStateIter {
        Self::NextStateIter { state: Some(self) }
    }
}

pub struct ByteChainNextStateIter<Iter: Iterator> {
    state: Option<ByteChain<Iter>>,
}

impl<Iter: Iterator> Iterator for ByteChainNextStateIter<Iter> {
    type Item = (Iter::Item, ByteChain<Iter>);

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
*/
