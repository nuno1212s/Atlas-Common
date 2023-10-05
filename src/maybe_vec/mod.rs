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
    One(T),
    Vec(Vec<T>)
}

impl<T> MaybeVec<T> {

    pub fn from_one(member: T) -> Self {
        Self::One(member)
    }

    pub fn from_many(objects: Vec<T>) -> Self {
        Self::Vec(objects)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        match self {
            MaybeVec::One(one) => {
                iter::once(one)
            }
            MaybeVec::Vec(vec) => {
                vec.iter()
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        match self {
            MaybeVec::One(one) => {
                iter::once(one)
            }
            MaybeVec::Vec(vec) => {
                vec.iter_mut()
            }
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        match self {
            MaybeVec::One(obj) => {
                iter::once(obj)
            }
            MaybeVec::Vec(vec) => {
                vec.into_iter()
            }
        }
    }

}