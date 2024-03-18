use std::collections::btree_set::{IntoIter, Iter};
use std::collections::BTreeSet;
use std::iter;
use std::iter::Once;

#[derive(Clone, Debug)]
pub enum MaybeOrderedVec<T> {
    None,
    One(T),
    Mult(BTreeSet<T>),
}

impl<T> MaybeOrderedVec<T> {
    pub fn builder() -> MaybeOrderedVecBuilder<T> {
        MaybeOrderedVecBuilder {
            current_value: Self::None,
        }
    }

    pub fn empty() -> Self {
        Self::None
    }

    pub fn from_one(member: T) -> Self {
        Self::One(member)
    }

    pub fn from_many(objects: Vec<T>) -> Self
    where
        T: Ord,
    {
        let mut result = BTreeSet::new();

        for obj in objects {
            result.insert(obj);
        }

        Self::Mult(result)
    }

    pub fn from_set(set: BTreeSet<T>) -> Self {
        Self::Mult(set)
    }

    pub fn len(&self) -> usize {
        match self {
            MaybeOrderedVec::None => 0,
            MaybeOrderedVec::One(_) => 1,
            MaybeOrderedVec::Mult(tree_set) => tree_set.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MaybeOrderedVec::None => true,
            MaybeOrderedVec::One(_) => false,
            MaybeOrderedVec::Mult(set) => set.is_empty(),
        }
    }

    pub fn iter(&self) -> ItRefMaybeVec<T> {
        match self {
            MaybeOrderedVec::One(one) => ItRefMaybeVec::One(iter::once(one)),
            MaybeOrderedVec::Mult(vec) => ItRefMaybeVec::Mult(vec.iter()),
            MaybeOrderedVec::None => ItRefMaybeVec::None,
        }
    }

    pub fn into_iter(self) -> ItMaybeVec<T> {
        match self {
            MaybeOrderedVec::One(one) => ItMaybeVec::One(iter::once(one)),
            MaybeOrderedVec::Mult(vec) => ItMaybeVec::Mult(vec.into_iter()),
            MaybeOrderedVec::None => ItMaybeVec::None,
        }
    }
}

pub enum ItMaybeVec<T> {
    None,
    One(Once<T>),
    Mult(IntoIter<T>),
}

impl<T> Iterator for ItMaybeVec<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ItMaybeVec::None => None,
            ItMaybeVec::One(iter) => iter.next(),
            ItMaybeVec::Mult(iter) => iter.next(),
        }
    }
}

impl<T> IntoIterator for MaybeOrderedVec<T> {
    type Item = T;
    type IntoIter = ItMaybeVec<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

pub enum ItRefMaybeVec<'a, T> {
    None,
    One(Once<&'a T>),
    Mult(Iter<'a, T>),
}

impl<'a, T> Iterator for ItRefMaybeVec<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ItRefMaybeVec::None => None,
            ItRefMaybeVec::One(iter) => iter.next(),
            ItRefMaybeVec::Mult(iter) => iter.next(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MaybeOrderedVecBuilder<T> {
    current_value: MaybeOrderedVec<T>,
}

impl<T> MaybeOrderedVecBuilder<T> {
    pub fn empty() -> Self {
        MaybeOrderedVecBuilder {
            current_value: MaybeOrderedVec::None,
        }
    }

    pub fn from_existing(vec: MaybeOrderedVec<T>) -> Self {
        Self { current_value: vec }
    }

    pub fn push(&mut self, value: T)
    where
        T: Ord,
    {
        let current = std::mem::replace(&mut self.current_value, MaybeOrderedVec::None);

        self.current_value = match current {
            MaybeOrderedVec::None => MaybeOrderedVec::One(value),
            MaybeOrderedVec::One(curr_value) => {
                let mut btree_set = BTreeSet::new();

                btree_set.insert(curr_value);
                btree_set.insert(value);

                MaybeOrderedVec::Mult(btree_set)
            }
            MaybeOrderedVec::Mult(mut vec) => {
                vec.insert(value);

                MaybeOrderedVec::Mult(vec)
            }
        };
    }

    pub fn build(self) -> MaybeOrderedVec<T> {
        self.current_value
    }
}
