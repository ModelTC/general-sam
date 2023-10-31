use std::{borrow::Cow, ops::Deref};

use super::treap::{NeedSwap, SplitTo, TreapNodeData, TreapTree};

pub trait RopeData: Clone {
    type TagType: Default;

    fn get_tag(&self) -> Option<Self::TagType>;
    fn reset_tag(&mut self);
    fn add_tag(&mut self, tag: Self::TagType) -> NeedSwap;

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>);
}

#[derive(Clone, Debug)]
pub struct RopeTreapData<Inner: RopeData> {
    inner: Inner,
    num: usize,
    rev_tag: bool,
}

impl<Inner: RopeData> RopeTreapData<Inner> {
    fn new(data: Inner) -> Self {
        Self {
            inner: data,
            num: 1,
            rev_tag: false,
        }
    }
}

impl<Inner: RopeData> Deref for RopeTreapData<Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Inner: RopeData> TreapNodeData for RopeTreapData<Inner> {
    type TagType = (bool, Option<Inner::TagType>);

    fn get_tag(&self) -> Option<Self::TagType> {
        match (self.rev_tag, self.inner.get_tag()) {
            (false, None) => None,
            other => Some(other),
        }
    }

    fn reset_tag(&mut self) {
        self.rev_tag = false;
        self.inner.reset_tag();
    }

    fn add_tag(&mut self, tag: Self::TagType) -> NeedSwap {
        self.rev_tag ^= tag.0;
        if let Some(inner_tag) = tag.1 {
            self.inner.add_tag(inner_tag) ^ tag.0
        } else {
            tag.0
        }
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.inner
            .update(left.map(|x| &x.inner), right.map(|x| &x.inner));
        self.num = left.map_or(0, |x| x.num) + right.map_or(0, |x| x.num) + 1;
    }
}

pub trait RopeBase: Sized + Clone {
    type InnerRopeData: RopeData;

    #[must_use]
    fn new(data: Self::InnerRopeData) -> Self;
    #[must_use]
    fn reverse(&self) -> Self;
    #[must_use]
    fn split(&self, num: usize) -> (Self, Self);
    #[must_use]
    fn merge(&self, other: &Self) -> Self;
    #[must_use]
    fn add_tag(&self, tag: <Self::InnerRopeData as RopeData>::TagType) -> Self;

    fn root_data_ref(&self) -> Option<&Self::InnerRopeData>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn for_each<F: FnMut(Self::InnerRopeData)>(&self, f: F);

    #[must_use]
    fn insert(&self, pos: usize, data: Self::InnerRopeData) -> Self {
        let (left, right) = self.split(pos);
        left.merge(&Self::new(data)).merge(&right)
    }

    #[must_use]
    fn remove(&self, pos: usize) -> (Self, Option<Self::InnerRopeData>) {
        if pos >= self.len() {
            return (self.clone(), None);
        }
        let (left, right) = self.split(pos);
        let (middle, right) = right.split(1);
        (left.merge(&right), middle.get(0))
    }

    #[must_use]
    fn push_back(&self, data: Self::InnerRopeData) -> Self {
        self.insert(self.len(), data)
    }

    #[must_use]
    fn push_front(&self, data: Self::InnerRopeData) -> Self {
        self.insert(0, data)
    }

    fn get(&self, pos: usize) -> Option<Self::InnerRopeData> {
        self.split(pos).1.split(1).0.root_data_ref().cloned()
    }
}

pub trait TreapBasedRopeBase:
    RopeBase
    + Deref<Target = TreapTree<RopeTreapData<Self::InnerRopeData>>>
    + From<TreapTree<RopeTreapData<Self::InnerRopeData>>>
{
    fn new_from_rng<R: FnMut() -> usize>(data: Self::InnerRopeData, rng: R) -> Self {
        TreapTree::new_from_rng(RopeTreapData::new(data), rng).into()
    }

    fn insert_from_rng<R: FnMut() -> usize>(
        &self,
        pos: usize,
        data: Self::InnerRopeData,
        rng: R,
    ) -> Self {
        let (left, right) = self.split(pos);
        left.merge(&Self::new_from_rng(data, rng)).merge(&right)
    }

    fn push_back_from_rng<R: FnMut() -> usize>(&self, data: Self::InnerRopeData, rng: R) -> Self {
        self.insert_from_rng(self.len(), data, rng)
    }

    fn push_front_from_rng<R: FnMut() -> usize>(&self, data: Self::InnerRopeData, rng: R) -> Self {
        self.insert_from_rng(0, data, rng)
    }

    fn query(&self, mut pos: usize) -> Option<Cow<RopeTreapData<Self::InnerRopeData>>> {
        self.deref().query(|node| {
            let left_size = node
                .get_left()
                .deref()
                .as_ref()
                .map_or(0, |left| left.data.num);
            let res = pos.cmp(&left_size);
            if pos > left_size {
                pos -= left_size + 1;
            }
            res
        })
    }
}

impl<
        InnerRopeData: RopeData,
        TreapBasedRope: From<TreapTree<RopeTreapData<InnerRopeData>>>
            + Deref<Target = TreapTree<RopeTreapData<InnerRopeData>>>
            + Clone,
    > TreapBasedRopeBase for TreapBasedRope
{
}
impl<
        InnerRopeData: RopeData,
        TreapBasedRope: From<TreapTree<RopeTreapData<InnerRopeData>>>
            + Deref<Target = TreapTree<RopeTreapData<InnerRopeData>>>
            + Clone,
    > RopeBase for TreapBasedRope
{
    type InnerRopeData = InnerRopeData;

    fn new(data: Self::InnerRopeData) -> Self {
        TreapTree::new(RopeTreapData::new(data)).into()
    }

    fn is_empty(&self) -> bool {
        self.is_some()
    }

    fn len(&self) -> usize {
        self.deref().root_data_ref().map_or(0, |x| x.num)
    }

    fn for_each<F: FnMut(Self::InnerRopeData)>(&self, mut f: F) {
        self.deref().for_each(&mut |x| f(x.inner))
    }

    fn reverse(&self) -> Self {
        self.deref().add_tag((true, None)).into()
    }

    fn split(&self, mut num: usize) -> (Self, Self) {
        let (u, v) = self.deref().split(|node| {
            let to_left_num = node.get_left().deref().as_ref().map_or(0, |x| x.data.num) + 1;
            if num >= to_left_num {
                num -= to_left_num;
                SplitTo::Left
            } else {
                SplitTo::Right
            }
        });
        (u.into(), v.into())
    }

    fn merge(&self, other: &Self) -> Self {
        self.deref().merge(other.deref()).into()
    }

    fn root_data_ref(&self) -> Option<&Self::InnerRopeData> {
        self.deref().root_data_ref().map(|x| &x.inner)
    }

    fn add_tag(&self, tag: <Self::InnerRopeData as RopeData>::TagType) -> Self {
        self.deref().add_tag((false, Some(tag))).into()
    }
}

#[derive(Clone, Debug)]
pub struct Rope<Inner: RopeData>(TreapTree<RopeTreapData<Inner>>);

impl<Inner: RopeData> From<TreapTree<RopeTreapData<Inner>>> for Rope<Inner> {
    fn from(value: TreapTree<RopeTreapData<Inner>>) -> Self {
        Self(value)
    }
}

impl<Inner: RopeData> Deref for Rope<Inner> {
    type Target = TreapTree<RopeTreapData<Inner>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Inner: RopeData> Default for Rope<Inner> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct RopeUntaggedInner<T: Clone>(T);

impl<T: Clone> RopeUntaggedInner<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Clone> Deref for RopeUntaggedInner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> From<T> for RopeUntaggedInner<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone> RopeData for RopeUntaggedInner<T> {
    type TagType = ();
    fn get_tag(&self) -> Option<Self::TagType> {
        None
    }
    fn reset_tag(&mut self) {}
    fn update(&mut self, _: Option<&Self>, _: Option<&Self>) {}
    fn add_tag(&mut self, _: Self::TagType) -> NeedSwap {
        false
    }
}

pub type UntaggedRope<T> = Rope<RopeUntaggedInner<T>>;

#[test]
fn test_rope() {
    let rope = UntaggedRope::<char>::default();
    assert!(rope.get(0).is_none());

    let rope = rope.push_front('a'.into());
    assert!(rope.get(0).is_some_and(|x| *x == 'a'));

    let rope = rope.push_front('b'.into());
    let rope = rope.push_back('c'.into());
    assert!(rope.get(0).is_some_and(|x| *x == 'b'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'c'));
    assert!(rope.get(3).is_none());
    assert!(rope.query(0).is_some_and(|x| ***x == 'b'));
    assert!(rope.query(1).is_some_and(|x| ***x == 'a'));
    assert!(rope.query(2).is_some_and(|x| ***x == 'c'));
    assert!(rope.query(3).is_none());

    let rope = rope.reverse();
    assert!(rope.get(0).is_some_and(|x| *x == 'c'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'b'));
    assert!(rope.get(3).is_none());

    let (l, r) = rope.split(1);
    assert!(rope.get(0).is_some_and(|x| *x == 'c'));
    assert!(rope.get(1).is_some_and(|x| *x == 'a'));
    assert!(rope.get(2).is_some_and(|x| *x == 'b'));
    assert!(rope.get(3).is_none());
    assert!(l.get(0).is_some_and(|x| *x == 'c'));
    assert!(l.get(1).is_none());
    assert!(r.get(0).is_some_and(|x| *x == 'a'));
    assert!(r.get(1).is_some_and(|x| *x == 'b'));
    assert!(r.get(2).is_none());

    let to_vec = |p: UntaggedRope<char>| -> Vec<char> {
        let mut res = Vec::<char>::new();
        p.for_each(&mut |d: RopeUntaggedInner<char>| res.push(*d));
        res
    };

    let reversed = rope.reverse();
    let v = to_vec(reversed);
    v.iter()
        .zip(['b', 'a', 'c'])
        .for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(rope);
    v.iter()
        .zip(['c', 'a', 'b'])
        .for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(l);
    v.iter().zip(['c']).for_each(|(i, j)| assert_eq!(*i, j));

    let v = to_vec(r);
    v.iter()
        .zip(['a', 'b'])
        .for_each(|(i, j)| assert_eq!(*i, j));
}
