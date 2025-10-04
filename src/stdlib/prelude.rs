pub use crate::stdlib::collections::vec::Vec;
pub use crate::stdlib::collections::hashmap::HashMap;
pub use crate::stdlib::collections::hashset::HashSet;
pub use crate::stdlib::string::operations::String;

pub trait Clone {
    fn clone(&self) -> Self;
}

pub trait Copy: Clone {}

pub trait Drop {
    fn drop(&mut self);
}

pub trait Eq: PartialEq<Self> {}

pub trait PartialEq<Rhs = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

pub trait Ord: Eq + PartialOrd<Self> {
    fn cmp(&self, other: &Self) -> Ordering;
    
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self >= other { self } else { other }
    }
    
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self <= other { self } else { other }
    }
}

pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
    
    fn lt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }
    
    fn le(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }
    
    fn gt(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }
    
    fn ge(&self, other: &Rhs) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Less,
    Equal,
    Greater,
}

pub trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H);
}

pub trait Hasher {
    fn finish(&self) -> u64;
    fn write(&mut self, bytes: &[u8]);
    
    fn write_u8(&mut self, i: u8) {
        self.write(&[i]);
    }
    
    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_i8(&mut self, i: i8) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_i16(&mut self, i: i16) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_i32(&mut self, i: i32) {
        self.write(&i.to_ne_bytes());
    }
    
    fn write_i64(&mut self, i: i64) {
        self.write(&i.to_ne_bytes());
    }
}

pub trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
    
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |count, _| count + 1)
    }
    
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut accum = init;
        while let Some(x) = self.next() {
            accum = f(accum, x);
        }
        accum
    }
    
    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        FromIterator::from_iter(self)
    }
    
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        Map { iter: self, f }
    }
    
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Filter { iter: self, predicate }
    }
}

pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    
    fn into_iter(self) -> Self::IntoIter;
}

pub trait FromIterator<A>: Sized {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self;
}

pub struct Map<I, F> {
    iter: I,
    f: F,
}

impl<B, I: Iterator, F: FnMut(I::Item) -> B> Iterator for Map<I, F> {
    type Item = B;
    
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(&mut self.f)
    }
}

pub struct Filter<I, P> {
    iter: I,
    predicate: P,
}

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for Filter<I, P> {
    type Item = I::Item;
    
    fn next(&mut self) -> Option<I::Item> {
        while let Some(item) = self.iter.next() {
            if (self.predicate)(&item) {
                return Some(item);
            }
        }
        None
    }
}

pub enum Option<T> {
    None,
    Some(T),
}

impl<T> Option<T> {
    pub fn is_some(&self) -> bool {
        matches!(*self, Some(_))
    }
    
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    
    pub fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Some(x) => x,
            None => default,
        }
    }
    
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(*self, Ok(_))
    }
    
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
    
    pub fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(x) => x,
            Err(_) => default,
        }
    }
    
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E> {
        match self {
            Ok(t) => Ok(f(t)),
            Err(e) => Err(e),
        }
    }
}

