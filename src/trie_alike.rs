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
