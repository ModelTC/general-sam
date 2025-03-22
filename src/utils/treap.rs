//! Persistent treap.

use std::{borrow::Cow, ops::Deref, sync::Arc};

use rand::random;

pub type NeedSwap = bool;

#[derive(Clone, Debug)]
pub enum SplitTo {
    Left,
    Right,
}

pub trait TreapNodeData: Clone {
    type TagType: Default;

    fn get_tag(&self) -> Option<Self::TagType>;
    fn reset_tag(&mut self);
    fn add_tag(&mut self, tag: Self::TagType) -> NeedSwap;
    fn update(&mut self, left: Option<&Self>, right: Option<&Self>);
}

#[derive(Clone, Debug)]
pub struct TreapTree<DataType: TreapNodeData>(Option<Arc<TreapNode<DataType>>>);

#[derive(Clone, Debug)]
pub struct TreapNode<DataType: TreapNodeData> {
    pub data: DataType,
    height: u64,
    _left: TreapTree<DataType>,
    _right: TreapTree<DataType>,
}

impl<DataType: TreapNodeData> TreapNode<DataType> {
    fn new(data: DataType) -> Self {
        Self {
            data,
            height: random(),
            _left: Default::default(),
            _right: Default::default(),
        }
    }

    fn new_from_rng<R: FnMut() -> u64>(data: DataType, mut rng: R) -> Self {
        Self {
            data,
            height: rng(),
            _left: Default::default(),
            _right: Default::default(),
        }
    }

    fn update(&mut self) {
        self.data.update(
            self._left.as_ref().map(|x| &x.data),
            self._right.as_ref().map(|x| &x.data),
        )
    }

    fn add_tag(&mut self, tag: DataType::TagType) {
        if self.data.add_tag(tag) {
            std::mem::swap(&mut self._left, &mut self._right)
        }
    }

    #[must_use]
    pub fn get_left(&self) -> Cow<TreapTree<DataType>> {
        match self.data.get_tag() {
            Some(tag) => Cow::Owned(self._left.add_tag(tag)),
            None => Cow::Borrowed(&self._left),
        }
    }

    #[must_use]
    pub fn get_right(&self) -> Cow<TreapTree<DataType>> {
        match self.data.get_tag() {
            Some(tag) => Cow::Owned(self._right.add_tag(tag)),
            None => Cow::Borrowed(&self._right),
        }
    }

    fn set_left(&mut self, left: TreapTree<DataType>) {
        if let Some(tag) = self.data.get_tag() {
            self._right = self._right.add_tag(tag);
        }
        self.data.reset_tag();
        self._left = left;
        self.update();
    }

    fn set_right(&mut self, right: TreapTree<DataType>) {
        if let Some(tag) = self.data.get_tag() {
            self._left = self._left.add_tag(tag);
        }
        self.data.reset_tag();
        self._right = right;
        self.update();
    }
}

impl<DataType: TreapNodeData> Default for TreapTree<DataType> {
    fn default() -> Self {
        Self(None)
    }
}

impl<DataType: TreapNodeData> Deref for TreapTree<DataType> {
    type Target = Option<Arc<TreapNode<DataType>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<DataType: TreapNodeData> TreapTree<DataType> {
    pub fn new(data: DataType) -> Self {
        Self(Some(Arc::new(TreapNode::new(data))))
    }

    pub fn new_from_rng<R: FnMut() -> u64>(data: DataType, rng: R) -> Self {
        Self(Some(Arc::new(TreapNode::new_from_rng(data, rng))))
    }

    pub fn root_data_ref(&self) -> Option<&DataType> {
        self.as_ref().map(|x| &x.data)
    }

    #[must_use]
    pub fn map<F: FnOnce(&mut TreapNode<DataType>)>(&self, f: F) -> Self {
        if let Some(node_ref) = self.deref() {
            let mut node = node_ref.deref().clone();
            f(&mut node);
            Self(Some(Arc::new(node)))
        } else {
            Self::default()
        }
    }

    #[must_use]
    pub fn add_tag(&self, tag: DataType::TagType) -> Self {
        self.map(|node| node.add_tag(tag))
    }

    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        match (self.deref(), other.deref()) {
            (None, None) => Self::default(),
            (None, Some(_)) => other.clone(),
            (Some(_), None) => self.clone(),
            (Some(left), Some(right)) => {
                if left.height > right.height {
                    let mut u = left.deref().to_owned();
                    u.set_right(u.get_right().merge(other));
                    Self(Some(Arc::new(u)))
                } else {
                    let mut v = right.deref().to_owned();
                    v.set_left(self.merge(&v.get_left()));
                    Self(Some(Arc::new(v)))
                }
            }
        }
    }

    #[must_use]
    pub fn split<F: FnMut(&mut TreapNode<DataType>) -> SplitTo>(&self, mut f: F) -> (Self, Self) {
        if let Some(node_ref) = self.deref() {
            let mut node = node_ref.deref().clone();
            match f(&mut node) {
                SplitTo::Left => {
                    let (left, right) = node.get_right().split(f);
                    node.set_right(left);
                    (Self(Some(Arc::new(node))), right)
                }
                SplitTo::Right => {
                    let (left, right) = node.get_left().split(f);
                    node.set_left(right);
                    (left, Self(Some(Arc::new(node))))
                }
            }
        } else {
            (Self::default(), Self::default())
        }
    }

    #[must_use]
    pub fn query<F: FnMut(&TreapNode<DataType>) -> std::cmp::Ordering>(
        &self,
        mut f: F,
    ) -> Option<Cow<DataType>> {
        if let Some(node_ref) = self.deref() {
            match f(node_ref) {
                std::cmp::Ordering::Equal => Some(Cow::Borrowed(&node_ref.data)),
                std::cmp::Ordering::Less => match node_ref.get_left() {
                    Cow::Borrowed(left) => left.query(f),
                    Cow::Owned(left) => left.query(f).map(|x| Cow::Owned(x.into_owned())),
                },
                std::cmp::Ordering::Greater => match node_ref.get_right() {
                    Cow::Borrowed(right) => right.query(f),
                    Cow::Owned(right) => right.query(f).map(|x| Cow::Owned(x.into_owned())),
                },
            }
        } else {
            None
        }
    }

    pub fn for_each<F: FnMut(DataType)>(&self, f: &mut F) {
        if let Some(node_ref) = self.deref() {
            node_ref.get_left().for_each(f);
            f(node_ref.data.clone());
            node_ref.get_right().for_each(f);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.is_none()
    }
}
