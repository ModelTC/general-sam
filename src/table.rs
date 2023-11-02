//! Transition table backends.

use std::collections::{btree_map, BTreeMap};

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
    type IterType<'a> = btree_map::Iter<'a, KeyType, GeneralSAMNodeID> where Self: 'a, Self::KeyType: 'a;

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
