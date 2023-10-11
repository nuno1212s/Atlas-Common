use std::collections::BTreeSet;
use std::iter;

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
    
    pub fn empty() -> Self { Self::None }
    
    pub fn from_one(member: T) -> Self {
        Self::One(member)
    }

    pub fn from_many(objects: Vec<T>) -> Self {
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
            MaybeOrderedVec::Mult(set) => set.is_empty()
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        match self {
            MaybeOrderedVec::One(one) => iter::once(one),
            MaybeOrderedVec::Mult(vec) => vec.iter(),
            MaybeOrderedVec::None => iter::empty()
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item=T> {
        match self {
            MaybeOrderedVec::One(one) => iter::once(one),
            MaybeOrderedVec::Mult(vec) => vec.into_iter(),
            MaybeOrderedVec::None => iter::empty()
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
        Self {
            current_value: vec,
        }
    }

    pub fn push(&mut self, value: T) {

        let current = std::mem::replace(&mut self.current_value, MaybeOrderedVec::None);

        self.current_value = match current {
            MaybeOrderedVec::None => {
                MaybeOrderedVec::One(value)
            }
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