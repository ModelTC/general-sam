//! Transition table backends.

use std::{
    collections::{BTreeMap, HashMap},
    iter::repeat_n,
    marker::PhantomData,
};

use crate::GeneralSamNodeID;

#[derive(Clone, Debug)]
pub struct WithKeyDerefedIter<
    'a,
    KeyType: 'a + Clone,
    IterType: Iterator<Item = (&'a KeyType, &'a GeneralSamNodeID)>,
> {
    inner: IterType,
}

impl<'a, KeyType: 'a + Clone, IterType: Iterator<Item = (&'a KeyType, &'a GeneralSamNodeID)>>
    Iterator for WithKeyDerefedIter<'a, KeyType, IterType>
{
    type Item = (KeyType, &'a GeneralSamNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (x.0.clone(), x.1))
    }
}

#[derive(Clone, Debug)]
pub struct TransitionIter<
    'a,
    KeyType: 'a,
    IterType: Iterator<Item = (KeyType, &'a GeneralSamNodeID)>,
> {
    inner: IterType,
}

impl<'a, KeyType: 'a, IterType: Iterator<Item = (KeyType, &'a GeneralSamNodeID)>> Iterator
    for TransitionIter<'a, KeyType, IterType>
{
    type Item = &'a GeneralSamNodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

pub trait TransitionTable {
    type KeyType: Clone;
    type IterType<'a>: Iterator<Item = (Self::KeyType, &'a GeneralSamNodeID)>
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (Self::KeyType, &'b GeneralSamNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b;
    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSamNodeID>;
    fn get_mut(&mut self, key: &Self::KeyType) -> Option<&mut GeneralSamNodeID>;
    fn iter(&self) -> Self::IterType<'_>;

    fn contains_key(&self, key: &Self::KeyType) -> bool {
        self.get(key).is_some()
    }

    fn transitions(&self) -> TransitionIter<'_, Self::KeyType, Self::IterType<'_>> {
        TransitionIter { inner: self.iter() }
    }
}

pub trait ConstructiveTransitionTable: TransitionTable + Clone + Default {
    fn insert(&mut self, key: Self::KeyType, trans: GeneralSamNodeID);

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (Self::KeyType, &'b GeneralSamNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        let mut res = Self::default();
        for (k, v) in iter {
            res.insert(k, *v);
        }
        res
    }
}

pub type BTreeTransTable<KeyType> = BTreeMap<KeyType, GeneralSamNodeID>;

impl<KeyType: Ord + Clone> ConstructiveTransitionTable for BTreeTransTable<KeyType> {
    fn insert(&mut self, key: KeyType, trans: GeneralSamNodeID) {
        BTreeMap::insert(self, key, trans);
    }
}

impl<KeyType: Clone + Ord> TransitionTable for BTreeTransTable<KeyType> {
    type KeyType = KeyType;
    type IterType<'a>
        = WithKeyDerefedIter<
        'a,
        KeyType,
        std::collections::btree_map::Iter<'a, KeyType, GeneralSamNodeID>,
    >
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSamNodeID> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSamNodeID> {
        BTreeMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        WithKeyDerefedIter {
            inner: BTreeMap::iter(self),
        }
    }

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (KeyType, &'b GeneralSamNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        <Self as ConstructiveTransitionTable>::from_kv_iter(iter)
    }
}

pub type HashTransTable<KeyType> = HashMap<KeyType, GeneralSamNodeID>;

impl<KeyType: std::hash::Hash + Eq + Clone> ConstructiveTransitionTable
    for HashTransTable<KeyType>
{
    fn insert(&mut self, key: KeyType, trans: GeneralSamNodeID) {
        HashMap::insert(self, key, trans);
    }
}

impl<KeyType: std::hash::Hash + Eq + Clone> TransitionTable for HashTransTable<KeyType> {
    type KeyType = KeyType;
    type IterType<'a>
        = WithKeyDerefedIter<
        'a,
        KeyType,
        std::collections::hash_map::Iter<'a, KeyType, GeneralSamNodeID>,
    >
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSamNodeID> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSamNodeID> {
        HashMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        WithKeyDerefedIter {
            inner: HashMap::iter(self),
        }
    }

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (KeyType, &'b GeneralSamNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        <Self as ConstructiveTransitionTable>::from_kv_iter(iter)
    }
}

fn bisect_unstable<K: Ord, V, C: AsRef<[(K, V)]>>(container: C, key: &K) -> Option<usize> {
    let (mut lo, mut hi) = (0, container.as_ref().len());
    while hi - lo > 0 {
        let mid = (lo + hi) / 2;
        match container.as_ref()[mid].0.cmp(key) {
            std::cmp::Ordering::Equal => {
                return Some(mid);
            }
            std::cmp::Ordering::Less => {
                lo = mid + 1;
            }
            std::cmp::Ordering::Greater => {
                hi = mid;
            }
        }
    }

    if lo < hi && container.as_ref()[lo].0 == *key {
        Some(lo)
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct BisectTable<
    K: Clone + Ord,
    C: AsRef<[(K, GeneralSamNodeID)]>
        + AsMut<[(K, GeneralSamNodeID)]>
        + FromIterator<(K, GeneralSamNodeID)>,
> {
    inner: C,
    phantom: PhantomData<K>,
}

#[derive(Clone, Debug)]
pub struct BisectTableIter<'s, K: Clone + Ord> {
    inner: core::slice::Iter<'s, (K, GeneralSamNodeID)>,
}

impl<'s, K: Clone + Ord> Iterator for BisectTableIter<'s, K> {
    type Item = (K, &'s GeneralSamNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (x.0.clone(), &x.1))
    }
}

impl<
    K: Clone + Ord,
    C: AsRef<[(K, GeneralSamNodeID)]>
        + AsMut<[(K, GeneralSamNodeID)]>
        + FromIterator<(K, GeneralSamNodeID)>,
> TransitionTable for BisectTable<K, C>
{
    type KeyType = K;
    type IterType<'a>
        = BisectTableIter<'a, K>
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSamNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &self.inner.as_ref()[i].1)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut GeneralSamNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &mut self.inner.as_mut()[i].1)
    }

    fn iter(&self) -> Self::IterType<'_> {
        BisectTableIter {
            inner: self.inner.as_ref().iter(),
        }
    }

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (K, &'b GeneralSamNodeID)>>(iter: Iter) -> Self
    where
        Self::KeyType: 'b,
    {
        let mut inner: Box<[(K, GeneralSamNodeID)]> =
            iter.into_iter().map(|(u, v)| (u.clone(), *v)).collect();
        inner.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Self {
            inner: inner.iter().map(|x| (x.0.clone(), x.1)).collect(),
            phantom: Default::default(),
        }
    }
}

pub type VecBisectTable<K> = BisectTable<K, Vec<(K, GeneralSamNodeID)>>;
pub type BoxBisectTable<K> = BisectTable<K, Box<[(K, GeneralSamNodeID)]>>;

pub trait SmallAlphabet: Copy + Ord + Into<usize> {
    const SIZE_LOG_2: usize;
    const SIZE: usize = 1 << Self::SIZE_LOG_2;

    fn from_usize(val: usize) -> Self;
}

impl SmallAlphabet for bool {
    const SIZE_LOG_2: usize = 1;

    fn from_usize(val: usize) -> Self {
        (val & 1) > 0
    }
}

impl SmallAlphabet for u8 {
    const SIZE_LOG_2: usize = 8;

    fn from_usize(val: usize) -> Self {
        (val & (Self::SIZE - 1)) as Self
    }
}

#[derive(Clone, Debug)]
pub struct WholeAlphabetTable<
    K: SmallAlphabet,
    C: AsRef<[Option<GeneralSamNodeID>]>
        + AsMut<[Option<GeneralSamNodeID>]>
        + FromIterator<Option<GeneralSamNodeID>>
        + Clone,
> {
    inner: C,
    phantom: PhantomData<K>,
}

#[derive(Clone, Debug)]
pub struct WholeAlphabetTableIter<'s, K: SmallAlphabet> {
    inner: std::iter::Enumerate<core::slice::Iter<'s, Option<GeneralSamNodeID>>>,
    phantom: PhantomData<K>,
}

impl<'s, K: SmallAlphabet> Iterator for WholeAlphabetTableIter<'s, K> {
    type Item = (K, &'s GeneralSamNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, v) in self.inner.by_ref() {
            if let Some(v) = v {
                return Some((K::from_usize(k), v));
            }
        }
        None
    }
}

impl<
    K: SmallAlphabet,
    C: AsRef<[Option<GeneralSamNodeID>]>
        + AsMut<[Option<GeneralSamNodeID>]>
        + FromIterator<Option<GeneralSamNodeID>>
        + Clone,
> Default for WholeAlphabetTable<K, C>
{
    fn default() -> Self {
        Self {
            inner: C::from_iter(repeat_n(None, K::SIZE)),
            phantom: Default::default(),
        }
    }
}

impl<
    K: SmallAlphabet,
    C: AsRef<[Option<GeneralSamNodeID>]>
        + AsMut<[Option<GeneralSamNodeID>]>
        + FromIterator<Option<GeneralSamNodeID>>
        + Clone,
> ConstructiveTransitionTable for WholeAlphabetTable<K, C>
{
    fn insert(&mut self, key: Self::KeyType, trans: GeneralSamNodeID) {
        let k: usize = key.into();
        self.inner.as_mut()[k] = Some(trans)
    }
}

impl<
    K: SmallAlphabet,
    C: AsRef<[Option<GeneralSamNodeID>]>
        + AsMut<[Option<GeneralSamNodeID>]>
        + FromIterator<Option<GeneralSamNodeID>>
        + Clone,
> TransitionTable for WholeAlphabetTable<K, C>
{
    type KeyType = K;
    type IterType<'a>
        = WholeAlphabetTableIter<'a, K>
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSamNodeID> {
        let k: usize = (*key).into();
        self.inner.as_ref().get(k).and_then(|x| x.as_ref())
    }

    fn get_mut(&mut self, key: &Self::KeyType) -> Option<&mut GeneralSamNodeID> {
        let k: usize = (*key).into();
        self.inner.as_mut().get_mut(k).and_then(|x| x.as_mut())
    }

    fn iter(&self) -> Self::IterType<'_> {
        WholeAlphabetTableIter {
            inner: self.inner.as_ref().iter().enumerate(),
            phantom: Default::default(),
        }
    }

    fn from_kv_iter<'b, Iter: IntoIterator<Item = (Self::KeyType, &'b GeneralSamNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        <Self as ConstructiveTransitionTable>::from_kv_iter(iter)
    }
}
