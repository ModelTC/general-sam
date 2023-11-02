//! Transition table backends.

use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::GeneralSAMNodeID;

pub struct TransitionIter<
    'a,
    KeyType: 'a,
    IterType: Iterator<Item = (&'a KeyType, &'a GeneralSAMNodeID)>,
> {
    inner: IterType,
}

impl<'a, KeyType: 'a, IterType: Iterator<Item = (&'a KeyType, &'a GeneralSAMNodeID)>> Iterator
    for TransitionIter<'a, KeyType, IterType>
{
    type Item = &'a GeneralSAMNodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

pub trait TransitionTable {
    type KeyType: Clone;
    type IterType<'a>: Iterator<Item = (&'a Self::KeyType, &'a GeneralSAMNodeID)>
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn from_kv_iter<'b, Iter: Iterator<Item = (&'b Self::KeyType, &'b GeneralSAMNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b;
    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSAMNodeID>;
    fn get_mut(&mut self, key: &Self::KeyType) -> Option<&mut GeneralSAMNodeID>;
    fn iter(&self) -> Self::IterType<'_>;

    fn contains_key(&self, key: &Self::KeyType) -> bool {
        self.get(key).is_some()
    }

    fn transitions(&self) -> TransitionIter<'_, Self::KeyType, Self::IterType<'_>> {
        TransitionIter { inner: self.iter() }
    }
}

pub trait ConstructiveTransitionTable: TransitionTable + Clone + Default {
    fn insert(&mut self, key: Self::KeyType, trans: GeneralSAMNodeID);

    fn from_kv_iter<'b, Iter: Iterator<Item = (&'b Self::KeyType, &'b GeneralSAMNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        let mut res = Self::default();
        for (k, v) in iter {
            res.insert(k.clone(), *v);
        }
        res
    }
}

pub type BTreeTransTable<KeyType> = BTreeMap<KeyType, GeneralSAMNodeID>;

impl<KeyType: Ord + Clone> ConstructiveTransitionTable for BTreeTransTable<KeyType> {
    fn insert(&mut self, key: KeyType, trans: GeneralSAMNodeID) {
        BTreeMap::insert(self, key, trans);
    }
}

impl<KeyType: Clone + Ord> TransitionTable for BTreeTransTable<KeyType> {
    type KeyType = KeyType;
    type IterType<'a> = std::collections::btree_map::Iter<'a, KeyType, GeneralSAMNodeID> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSAMNodeID> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSAMNodeID> {
        BTreeMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        BTreeMap::iter(self)
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (&'b KeyType, &'b GeneralSAMNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        <Self as ConstructiveTransitionTable>::from_kv_iter(iter)
    }
}

pub type HashTransTable<KeyType> = HashMap<KeyType, GeneralSAMNodeID>;

impl<KeyType: std::hash::Hash + Eq + Clone> ConstructiveTransitionTable
    for HashTransTable<KeyType>
{
    fn insert(&mut self, key: KeyType, trans: GeneralSAMNodeID) {
        HashMap::insert(self, key, trans);
    }
}

impl<KeyType: std::hash::Hash + Eq + Clone> TransitionTable for HashTransTable<KeyType> {
    type KeyType = KeyType;
    type IterType<'a> = std::collections::hash_map::Iter<'a, KeyType, GeneralSAMNodeID> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSAMNodeID> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSAMNodeID> {
        HashMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        HashMap::iter(self)
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (&'b KeyType, &'b GeneralSAMNodeID)>>(
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

pub struct BisectTable<
    K: Clone + Ord,
    C: AsRef<[(K, GeneralSAMNodeID)]>
        + AsMut<[(K, GeneralSAMNodeID)]>
        + FromIterator<(K, GeneralSAMNodeID)>,
> {
    inner: C,
    phantom: PhantomData<K>,
}

pub struct SliceTableIter<'s, K: Clone + Ord> {
    inner: core::slice::Iter<'s, (K, GeneralSAMNodeID)>,
}

impl<'s, K: Clone + Ord> Iterator for SliceTableIter<'s, K> {
    type Item = (&'s K, &'s GeneralSAMNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (&x.0, &x.1))
    }
}

impl<
        K: Clone + Ord,
        C: AsRef<[(K, GeneralSAMNodeID)]>
            + AsMut<[(K, GeneralSAMNodeID)]>
            + FromIterator<(K, GeneralSAMNodeID)>,
    > TransitionTable for BisectTable<K, C>
{
    type KeyType = K;
    type IterType<'a> = SliceTableIter<'a, K> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSAMNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &self.inner.as_ref()[i].1)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut GeneralSAMNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &mut self.inner.as_mut()[i].1)
    }

    fn iter(&self) -> Self::IterType<'_> {
        SliceTableIter {
            inner: self.inner.as_ref().iter(),
        }
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (&'b K, &'b GeneralSAMNodeID)>>(iter: Iter) -> Self
    where
        Self::KeyType: 'b,
    {
        let mut inner: Box<[(K, GeneralSAMNodeID)]> = iter.map(|(u, v)| (u.clone(), *v)).collect();
        inner.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Self {
            inner: inner.iter().map(|x| (x.0.clone(), x.1)).collect(),
            phantom: Default::default(),
        }
    }
}

pub type VecBisectTable<K> = BisectTable<K, Vec<(K, GeneralSAMNodeID)>>;
pub type BoxBisectTable<K> = BisectTable<K, Box<[(K, GeneralSAMNodeID)]>>;
