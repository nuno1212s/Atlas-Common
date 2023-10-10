use std::collections::BTreeSet;
use std::iter;

pub mod ordered;

/// Utility type for when we want to have a type
/// which can either be a single item, or a vec of items
/// we used this to avoid heap allocations with the overuse of vecs
/// when we only want a single item.
///
/// With this enum, we can represent both options while still maintaining the
/// speed and efficiency of stack allocation when we have a single value
#[derive(Clone, Debug)]
pub enum MaybeVec<T> {
    None,
    One(T),
    Mult(Vec<T>),
}

impl<T> MaybeVec<T> {
    pub fn from_one(member: T) -> Self {
        Self::One(member)
    }

    pub fn from_many(objects: Vec<T>) -> Self {
        Self::Mult(objects)
    }

    pub fn builder() -> MaybeVecBuilder<T> {
        MaybeVecBuilder::empty()
    }

    pub fn len(&self) -> usize {
        match self {
            MaybeVec::One(_) => 1,
            MaybeVec::Mult(vec) => vec.len(),
            MaybeVec::None => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MaybeVec::One(_) => false,
            MaybeVec::Mult(vec) => vec.is_empty(),
            MaybeVec::None => true,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        match self {
            MaybeVec::One(one) => {
                iter::once(one)
            }
            MaybeVec::Mult(vec) => {
                vec.iter()
            }
            MaybeVec::None => {
                iter::empty()
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> {
        match self {
            MaybeVec::One(one) => {
                iter::once(one)
            }
            MaybeVec::Mult(vec) => {
                vec.iter_mut()
            }
            MaybeVec::None => {
                iter::empty()
            }
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item=T> {
        match self {
            MaybeVec::One(obj) => {
                iter::once(obj)
            }
            MaybeVec::Mult(vec) => {
                vec.into_iter()
            }
            MaybeVec::None => {
                iter::empty()
            }
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            MaybeVec::None => {
                Vec::new()
            }
            MaybeVec::One(val) => {
                vec![val]
            }
            MaybeVec::Mult(vec) => {
                vec
            }
        }
    }

    /// Join two maybe vecs
    pub fn joining(self, other: Self) -> Self {
        match self {
            MaybeVec::None => {
                other
            }
            MaybeVec::One(value) => {
                match other {
                    MaybeVec::None => {
                        MaybeVec::One(value)
                    }
                    MaybeVec::One(other) => {
                        MaybeVec::Mult(vec![value, other])
                    }
                    MaybeVec::Mult(mut other_vec) => {
                        other_vec.push(value);

                        MaybeVec::Mult(other_vec)
                    }
                }
            }
            MaybeVec::Mult(mut vec) => {
                match other {
                    MaybeVec::None => {
                        MaybeVec::Mult(vec)
                    }
                    MaybeVec::One(value) => {
                        vec.push(value)
                    }
                    MaybeVec::Mult(mut other) => {
                        vec.append(&mut other);

                        MaybeVec::Mult(vec)
                    }
                }
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct MaybeVecBuilder<T> {
    current_value: MaybeVec<T>,
}

impl<T> MaybeVecBuilder<T> {
    pub fn empty() -> Self {
        Self {
            current_value: MaybeVec::None
        }
    }

    pub fn from_existing(value: MaybeVec<T>) -> Self {
        Self {
            current_value: value
        }
    }

    pub fn push(&mut self, value: T) {
        let current = std::mem::replace(&mut self.current_value, MaybeVec::None);

        self.current_value = match current {
            MaybeVec::None => {
                MaybeVec::One(value)
            }
            MaybeVec::One(curr_value) => {
                MaybeVec::Mult(vec![curr_value, value])
            }
            MaybeVec::Mult(mut vec) => {
                vec.push(value);

                MaybeVec::Mult(vec)
            }
        };
    }

    pub fn build(self) -> MaybeVec<T> {
        self.current_value
    }
}