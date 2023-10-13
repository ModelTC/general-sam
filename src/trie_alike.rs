pub trait TrieNodeAlike<T, Iter>
where
    T: Ord,
    Iter: Iterator<Item = (T, Self)>,
{
    fn is_accepting(&self) -> bool;
    fn next_states(self) -> Iter;
}

pub struct ByteChain<S: AsRef<[u8]>> {
    string: S,
    pos: usize,
}

pub struct ByteChainIter<S: AsRef<[u8]>> {
    c: Option<ByteChain<S>>,
}

impl<S: AsRef<[u8]>> From<S> for ByteChain<S> {
    fn from(s: S) -> Self {
        ByteChain { string: s, pos: 0 }
    }
}

impl<S: AsRef<[u8]>> TrieNodeAlike<u8, ByteChainIter<S>> for ByteChain<S> {
    fn is_accepting(&self) -> bool {
        self.pos >= self.string.as_ref().len()
    }

    fn next_states(self) -> ByteChainIter<S> {
        ByteChainIter { c: Some(self) }
    }
}

impl<S: AsRef<[u8]>> Iterator for ByteChainIter<S> {
    type Item = (u8, ByteChain<S>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut res) = self.c.take() {
            return if res.pos < res.string.as_ref().len() {
                let c = &res.string.as_ref()[res.pos];
                res.pos += 1;
                Some((*c, res))
            } else {
                None
            };
        }
        None
    }
}
