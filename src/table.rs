//! Transition table backends.

use std::{
    collections::{BTreeMap, HashMap},
    iter::repeat,
    marker::PhantomData,
};

use crate::GeneralSAMNodeID;

pub struct WithKeyDerefedIter<
    'a,
    KeyType: 'a + Clone,
    IterType: Iterator<Item = (&'a KeyType, &'a GeneralSAMNodeID)>,
> {
    inner: IterType,
}

impl<'a, KeyType: 'a + Clone, IterType: Iterator<Item = (&'a KeyType, &'a GeneralSAMNodeID)>>
    Iterator for WithKeyDerefedIter<'a, KeyType, IterType>
{
    type Item = (KeyType, &'a GeneralSAMNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (x.0.clone(), x.1))
    }
}

pub struct TransitionIter<
    'a,
    KeyType: 'a,
    IterType: Iterator<Item = (KeyType, &'a GeneralSAMNodeID)>,
> {
    inner: IterType,
}

impl<'a, KeyType: 'a, IterType: Iterator<Item = (KeyType, &'a GeneralSAMNodeID)>> Iterator
    for TransitionIter<'a, KeyType, IterType>
{
    type Item = &'a GeneralSAMNodeID;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

pub trait TransitionTable {
    type KeyType: Clone;
    type IterType<'a>: Iterator<Item = (Self::KeyType, &'a GeneralSAMNodeID)>
    where
        Self: 'a,
        Self::KeyType: 'a;

    fn from_kv_iter<'b, Iter: Iterator<Item = (Self::KeyType, &'b GeneralSAMNodeID)>>(
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

    fn from_kv_iter<'b, Iter: Iterator<Item = (Self::KeyType, &'b GeneralSAMNodeID)>>(
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

pub type BTreeTransTable<KeyType> = BTreeMap<KeyType, GeneralSAMNodeID>;

impl<KeyType: Ord + Clone> ConstructiveTransitionTable for BTreeTransTable<KeyType> {
    fn insert(&mut self, key: KeyType, trans: GeneralSAMNodeID) {
        BTreeMap::insert(self, key, trans);
    }
}

impl<KeyType: Clone + Ord> TransitionTable for BTreeTransTable<KeyType> {
    type KeyType = KeyType;
    type IterType<'a> = WithKeyDerefedIter<'a, KeyType, std::collections::btree_map::Iter<'a, KeyType, GeneralSAMNodeID>> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSAMNodeID> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSAMNodeID> {
        BTreeMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        WithKeyDerefedIter {
            inner: BTreeMap::iter(self),
        }
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (KeyType, &'b GeneralSAMNodeID)>>(iter: Iter) -> Self
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
    type IterType<'a> = WithKeyDerefedIter<'a, KeyType, std::collections::hash_map::Iter<'a, KeyType, GeneralSAMNodeID>> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &KeyType) -> Option<&GeneralSAMNodeID> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &KeyType) -> Option<&mut GeneralSAMNodeID> {
        HashMap::get_mut(self, key)
    }

    fn iter(&self) -> Self::IterType<'_> {
        WithKeyDerefedIter {
            inner: HashMap::iter(self),
        }
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (KeyType, &'b GeneralSAMNodeID)>>(iter: Iter) -> Self
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

pub struct BisectTableIter<'s, K: Clone + Ord> {
    inner: core::slice::Iter<'s, (K, GeneralSAMNodeID)>,
}

impl<'s, K: Clone + Ord> Iterator for BisectTableIter<'s, K> {
    type Item = (K, &'s GeneralSAMNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| (x.0.clone(), &x.1))
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
    type IterType<'a> = BisectTableIter<'a, K> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSAMNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &self.inner.as_ref()[i].1)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut GeneralSAMNodeID> {
        bisect_unstable(&self.inner, key).map(|i| &mut self.inner.as_mut()[i].1)
    }

    fn iter(&self) -> Self::IterType<'_> {
        BisectTableIter {
            inner: self.inner.as_ref().iter(),
        }
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (K, &'b GeneralSAMNodeID)>>(iter: Iter) -> Self
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

#[derive(Clone)]
pub struct WholeAlphabetTable<
    K: SmallAlphabet,
    C: AsRef<[Option<GeneralSAMNodeID>]>
        + AsMut<[Option<GeneralSAMNodeID>]>
        + FromIterator<Option<GeneralSAMNodeID>>
        + Clone,
> {
    inner: C,
    phantom: PhantomData<K>,
}

pub struct WholeAlphabetTableIter<'s, K: SmallAlphabet> {
    inner: std::iter::Enumerate<core::slice::Iter<'s, Option<GeneralSAMNodeID>>>,
    phantom: PhantomData<K>,
}

impl<'s, K: SmallAlphabet> Iterator for WholeAlphabetTableIter<'s, K> {
    type Item = (K, &'s GeneralSAMNodeID);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, ref v) in self.inner.by_ref() {
            if let Some(ref v) = v {
                return Some((K::from_usize(k), v));
            }
        }
        None
    }
}

impl<
        K: SmallAlphabet,
        C: AsRef<[Option<GeneralSAMNodeID>]>
            + AsMut<[Option<GeneralSAMNodeID>]>
            + FromIterator<Option<GeneralSAMNodeID>>
            + Clone,
    > Default for WholeAlphabetTable<K, C>
{
    fn default() -> Self {
        Self {
            inner: C::from_iter(repeat(None).take(K::SIZE)),
            phantom: Default::default(),
        }
    }
}

impl<
        K: SmallAlphabet,
        C: AsRef<[Option<GeneralSAMNodeID>]>
            + AsMut<[Option<GeneralSAMNodeID>]>
            + FromIterator<Option<GeneralSAMNodeID>>
            + Clone,
    > ConstructiveTransitionTable for WholeAlphabetTable<K, C>
{
    fn insert(&mut self, key: Self::KeyType, trans: GeneralSAMNodeID) {
        let k: usize = key.into();
        self.inner.as_mut()[k] = Some(trans)
    }
}

impl<
        K: SmallAlphabet,
        C: AsRef<[Option<GeneralSAMNodeID>]>
            + AsMut<[Option<GeneralSAMNodeID>]>
            + FromIterator<Option<GeneralSAMNodeID>>
            + Clone,
    > TransitionTable for WholeAlphabetTable<K, C>
{
    type KeyType = K;
    type IterType<'a> = WholeAlphabetTableIter<'a, K> where Self: 'a, Self::KeyType: 'a;

    fn get(&self, key: &Self::KeyType) -> Option<&GeneralSAMNodeID> {
        let k: usize = (*key).into();
        self.inner.as_ref().get(k).and_then(|x| x.as_ref())
    }

    fn get_mut(&mut self, key: &Self::KeyType) -> Option<&mut GeneralSAMNodeID> {
        let k: usize = (*key).into();
        self.inner.as_mut().get_mut(k).and_then(|x| x.as_mut())
    }

    fn iter(&self) -> Self::IterType<'_> {
        WholeAlphabetTableIter {
            inner: self.inner.as_ref().iter().enumerate(),
            phantom: Default::default(),
        }
    }

    fn from_kv_iter<'b, Iter: Iterator<Item = (Self::KeyType, &'b GeneralSAMNodeID)>>(
        iter: Iter,
    ) -> Self
    where
        Self::KeyType: 'b,
    {
        <Self as ConstructiveTransitionTable>::from_kv_iter(iter)
    }
}

// not really fast
//
// pub trait Decomposable<To: SmallAlphabet>: Clone {
//     const SIZE_LOG_2: usize = 8 * std::mem::size_of::<Self>();
//     type IterType<'a>: Iterator<Item = To>
//     where
//         Self: 'a;
//
//     fn decompose(&self) -> Self::IterType<'_>;
// }
//
// pub struct DecomposeIntoBytesIter<'a, A: Decomposable<u8>> {
//     val: &'a A,
//     pos: usize,
// }
//
// impl<'a, A: Decomposable<u8>> Iterator for DecomposeIntoBytesIter<'a, A> {
//     type Item = u8;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.pos * 8 >= A::SIZE_LOG_2 {
//             return None;
//         }
//         let val = unsafe {
//             let ptr = (self.val as *const A as *const u8).add(self.pos);
//             *ptr
//         };
//         self.pos += 1;
//         Some(val)
//     }
// }
//
// impl<A: Clone> Decomposable<u8> for A {
//     type IterType<'a> = DecomposeIntoBytesIter<'a, Self> where A: 'a;
//
//     fn decompose(&self) -> Self::IterType<'_> {
//         DecomposeIntoBytesIter { val: self, pos: 0 }
//     }
// }
//
// #[cfg(feature = "trie")]
// #[cfg_attr(doc_cfg, doc(cfg(feature = "trie")))]
// pub use trie::{CharAsBytesTrieTransTable, TrieTransTable, TrieTransTableIter};
//
// #[cfg(feature = "trie")]
// #[cfg_attr(doc_cfg, doc(cfg(feature = "trie")))]
// mod trie {
//     use std::marker::PhantomData;
//
//     use crate::{
//         BTreeTransTable, ConstructiveTransitionTable, GeneralSAMNodeID, TransitionTable, Trie,
//         TrieNodeID, TRIE_NIL_NODE_ID,
//     };
//
//     use super::{Decomposable, SmallAlphabet, WholeAlphabetTable};
//
//     #[derive(Clone)]
//     pub struct TrieTransTable<
//         To: SmallAlphabet,
//         Key: Decomposable<To>,
//         Backend: TransitionTable<KeyType = To>,
//     > {
//         trie: Trie<Backend>,
//         values: Vec<Option<(Key, GeneralSAMNodeID)>>,
//         phantom: PhantomData<Key>,
//     }
//
//     pub struct TrieTransTableIter<'s, To: SmallAlphabet, Key: Decomposable<To>> {
//         inner: core::slice::Iter<'s, Option<(Key, GeneralSAMNodeID)>>,
//         phantom: PhantomData<To>,
//     }
//
//     impl<'s, To: SmallAlphabet, Key: Decomposable<To>> Iterator for TrieTransTableIter<'s, To, Key> {
//         type Item = (Key, &'s GeneralSAMNodeID);
//
//         fn next(&mut self) -> Option<Self::Item> {
//             for item in self.inner.by_ref() {
//                 if let Some((k, v)) = item.as_ref() {
//                     return Some((k.clone(), v));
//                 }
//             }
//             None
//         }
//     }
//
//     impl<To: SmallAlphabet, Key: Decomposable<To>, Backend: TransitionTable<KeyType = To>>
//         TrieTransTable<To, Key, Backend>
//     {
//         fn get_node_id(&self, key: &Key) -> TrieNodeID {
//             let mut state = self.trie.get_root_state();
//             for k in key.decompose() {
//                 state.goto(&k);
//             }
//             state.node_id
//         }
//
//         fn alter_backend_into<NewBackend: TransitionTable<KeyType = To>>(
//             self,
//         ) -> TrieTransTable<To, Key, NewBackend> {
//             TrieTransTable {
//                 trie: self.trie.alter_trans_table(),
//                 values: self.values,
//                 phantom: Default::default(),
//             }
//         }
//     }
//
//     impl<
//             To: SmallAlphabet,
//             Key: Decomposable<To>,
//             Backend: ConstructiveTransitionTable<KeyType = To>,
//         > Default for TrieTransTable<To, Key, Backend>
//     {
//         fn default() -> Self {
//             Self {
//                 trie: Default::default(),
//                 values: Default::default(),
//                 phantom: Default::default(),
//             }
//         }
//     }
//
//     impl<
//             To: SmallAlphabet,
//             Key: Decomposable<To>,
//             Backend: ConstructiveTransitionTable<KeyType = To>,
//         > ConstructiveTransitionTable for TrieTransTable<To, Key, Backend>
//     {
//         fn insert(&mut self, key: Self::KeyType, trans: GeneralSAMNodeID) {
//             let node_id = self.trie.insert_iter(key.decompose());
//             if self.values.len() <= node_id {
//                 self.values.resize(node_id + 1, None)
//             }
//             self.values[node_id] = Some((key, trans))
//         }
//     }
//
//     impl<To: SmallAlphabet, Key: Decomposable<To>, Backend: TransitionTable<KeyType = To>>
//         TransitionTable for TrieTransTable<To, Key, Backend>
//     {
//         type KeyType = Key;
//
//         type IterType<'a> = TrieTransTableIter<'a, To, Key> where To: 'a, Key: 'a, Backend: 'a;
//
//         fn from_kv_iter<'b, Iter: Iterator<Item = (Self::KeyType, &'b crate::GeneralSAMNodeID)>>(
//             iter: Iter,
//         ) -> Self
//         where
//             Self::KeyType: 'b,
//         {
//             <TrieTransTable<_, _, BTreeTransTable<_>> as ConstructiveTransitionTable>::from_kv_iter(
//                 iter,
//             )
//             .alter_backend_into()
//         }
//
//         fn get(&self, key: &Self::KeyType) -> Option<&crate::GeneralSAMNodeID> {
//             let node_id = self.get_node_id(key);
//             if node_id == TRIE_NIL_NODE_ID {
//                 return None;
//             }
//             self.values
//                 .get(node_id)
//                 .and_then(|x| x.as_ref().map(|x| &x.1))
//         }
//
//         fn get_mut(&mut self, key: &Self::KeyType) -> Option<&mut crate::GeneralSAMNodeID> {
//             let node_id = self.get_node_id(key);
//             if node_id == TRIE_NIL_NODE_ID {
//                 return None;
//             }
//             self.values
//                 .get_mut(node_id)
//                 .and_then(|x| x.as_mut().map(|x| &mut x.1))
//         }
//
//         fn iter(&self) -> Self::IterType<'_> {
//             Self::IterType {
//                 inner: self.values.iter(),
//                 phantom: PhantomData,
//             }
//         }
//     }
//
//     pub type CharAsBytesTrieTransTable<SeqBackend> =
//         TrieTransTable<u8, char, WholeAlphabetTable<u8, SeqBackend>>;
// }
