use std::collections::BTreeSet;
use std::iter;

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
    Vec(Vec<T>),
}

impl<T> MaybeVec<T> {
    pub fn from_one(member: T) -> Self {
        Self::One(member)
    }

    pub fn from_many(objects: Vec<T>) -> Self {
        Self::Vec(objects)
    }

    pub fn builder() -> MaybeVecBuilder<T> {
        MaybeVecBuilder::empty()
    }

    pub fn len(&self) -> usize {
        match self {
            MaybeVec::One(_) => {
                1
            }
            MaybeVec::Vec(vec) => {
                vec.len()
            }
            MaybeVec::None => {
                0
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MaybeVec::One(_) => {
                false
            }
            MaybeVec::Vec(vec) => {
                vec.is_empty()
            }
            MaybeVec::None => {
                true
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        match self {
            MaybeVec::One(one) => {
                iter::once(one)
            }
            MaybeVec::Vec(vec) => {
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
            MaybeVec::Vec(vec) => {
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
            MaybeVec::Vec(vec) => {
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
            MaybeVec::Vec(vec) => {
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
                        MaybeVec::Vec(vec![value, other])
                    }
                    MaybeVec::Vec(mut other_vec) => {
                        other_vec.push(value);

                        MaybeVec::Vec(other_vec)
                    }
                }
            }
            MaybeVec::Vec(mut vec) => {
                match other {
                    MaybeVec::None => {
                        MaybeVec::Vec(vec)
                    }
                    MaybeVec::One(value) => {
                        vec.push(value)
                    }
                    MaybeVec::Vec(mut other) => {
                        vec.append(&mut other);

                        MaybeVec::Vec(vec)
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum MaybeOrderedVec<T> {
    None,
    One(T),
    Multiple(BTreeSet<T>)
}

impl<T> MaybeOrderedVec<T> {

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

    pub fn push(&mut self, value: T) {
        let current = std::mem::replace(&mut self.current_value, MaybeVec::None);

        self.current_value = match current {
            MaybeVec::None => {
                MaybeVec::One(value)
            }
            MaybeVec::One(curr_value) => {
                MaybeVec::Vec(vec![curr_value, value])
            }
            MaybeVec::Vec(mut vec) => {
                vec.push(value);

                MaybeVec::Vec(vec)
            }
        };
    }

    pub fn build(self) -> MaybeVec<T> {
        self.current_value
    }
}