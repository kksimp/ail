//! List storage (ADR 0760). A `List[T]` of `int`, `float` or `bool` is kept packed, one machine word
//! (one byte for `bool`) per element, in place of one 32-byte `Value`; every other list stays boxed.
//! The representation is invisible to AIL code: a packed list and a boxed list with the same elements
//! behave identically (own.val.1: copies are `Rc` clones, copy on write).
//!
//! The kind is never taken from a static type. A packed list holds only the one variant (`Int`, `Float`
//! or `Bool`) it was built from; storing anything else, or any operation without a packed form, turns it
//! into a boxed list in place (`boxed_mut`), so a mismatch costs speed and never correctness.

use super::Value;
use std::borrow::Cow;
use std::sync::OnceLock;

/// Whether lists are built packed. `AIL_NO_PACK=1` keeps every list boxed: the oracle for the differential
/// tests (a program prints the same with and without packing).
fn packing() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("AIL_NO_PACK").is_none())
}

#[derive(Debug, Clone)]
pub(crate) enum List {
    Boxed(Vec<Value>),
    Bool(Vec<u8>),
    Int(Vec<i64>),
    Float(Vec<f64>),
}

impl Default for List {
    fn default() -> List {
        List::Boxed(Vec::new())
    }
}

impl List {
    /// A boxed list: no scan, the representation of every list not built by `pack`.
    #[inline]
    pub(crate) fn boxed(items: Vec<Value>) -> List {
        List::Boxed(items)
    }

    /// The packed form when every element is an `int`, a `float` or a `bool` (and the list is not empty),
    /// else the boxed one.
    pub(crate) fn pack(items: Vec<Value>) -> List {
        if !packing() {
            return List::Boxed(items);
        }
        match items.first() {
            Some(Value::Int(_)) if items.iter().all(|v| matches!(v, Value::Int(_))) => List::Int(
                items.iter().map(|v| if let Value::Int(x) = v { *x } else { 0 }).collect(),
            ),
            Some(Value::Float(_)) if items.iter().all(|v| matches!(v, Value::Float(_))) => List::Float(
                items.iter().map(|v| if let Value::Float(x) = v { *x } else { 0.0 }).collect(),
            ),
            Some(Value::Bool(_)) if items.iter().all(|v| matches!(v, Value::Bool(_))) => List::Bool(
                items.iter().map(|v| matches!(v, Value::Bool(true)) as u8).collect(),
            ),
            _ => List::Boxed(items),
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        match self {
            List::Boxed(v) => v.len(),
            List::Bool(v) => v.len(),
            List::Int(v) => v.len(),
            List::Float(v) => v.len(),
        }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The element at `i`, or `None` past the end.
    #[inline]
    pub(crate) fn get(&self, i: usize) -> Option<Value> {
        match self {
            List::Boxed(v) => v.get(i).cloned(),
            List::Bool(v) => v.get(i).map(|b| Value::Bool(*b != 0)),
            List::Int(v) => v.get(i).map(|x| Value::Int(*x)),
            List::Float(v) => v.get(i).map(|x| Value::Float(*x)),
        }
    }

    /// The element at `i`, which the caller has checked is in range.
    #[inline]
    pub(crate) fn at(&self, i: usize) -> Value {
        self.get(i).expect("checked: index in range")
    }

    pub(crate) fn first(&self) -> Option<Value> {
        self.get(0)
    }

    pub(crate) fn last(&self) -> Option<Value> {
        self.len().checked_sub(1).and_then(|i| self.get(i))
    }

    /// The elements as a slice: borrowed for a boxed list, built for a packed one.
    #[inline]
    pub(crate) fn slice(&self) -> Cow<'_, [Value]> {
        match self {
            List::Boxed(v) => Cow::Borrowed(v),
            _ => Cow::Owned(self.to_vec()),
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<Value> {
        match self {
            List::Boxed(v) => v.clone(),
            List::Bool(v) => v.iter().map(|b| Value::Bool(*b != 0)).collect(),
            List::Int(v) => v.iter().map(|x| Value::Int(*x)).collect(),
            List::Float(v) => v.iter().map(|x| Value::Float(*x)).collect(),
        }
    }

    pub(crate) fn into_vec(self) -> Vec<Value> {
        match self {
            List::Boxed(v) => v,
            other => other.to_vec(),
        }
    }

    /// The elements in order, as owned values.
    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = Value> + ExactSizeIterator + '_ {
        (0..self.len()).map(move |i| self.at(i))
    }

    /// The boxed vector, turning a packed list into a boxed one first. For the operations without a
    /// packed form.
    pub(crate) fn boxed_mut(&mut self) -> &mut Vec<Value> {
        if !matches!(self, List::Boxed(_)) {
            *self = List::Boxed(self.to_vec());
        }
        let List::Boxed(v) = self else { unreachable!("boxed above") };
        v
    }

    /// Replace the element at `i` (in range).
    #[inline]
    pub(crate) fn set(&mut self, i: usize, x: Value) {
        match (&mut *self, &x) {
            (List::Int(v), Value::Int(n)) => v[i] = *n,
            (List::Float(v), Value::Float(n)) => v[i] = *n,
            (List::Bool(v), Value::Bool(b)) => v[i] = *b as u8,
            (List::Boxed(v), _) => v[i] = x,
            _ => self.boxed_mut()[i] = x,
        }
    }

    pub(crate) fn push(&mut self, x: Value) {
        match (&mut *self, &x) {
            (List::Int(v), Value::Int(n)) => v.push(*n),
            (List::Float(v), Value::Float(n)) => v.push(*n),
            (List::Bool(v), Value::Bool(b)) => v.push(*b as u8),
            (List::Boxed(v), _) if !v.is_empty() => v.push(x),
            // The first element of an empty list picks the packed form.
            (List::Boxed(_), Value::Int(_) | Value::Float(_) | Value::Bool(_)) if !packing() => {
                self.boxed_mut().push(x)
            }
            (List::Boxed(_), Value::Int(n)) => *self = List::Int(vec![*n]),
            (List::Boxed(_), Value::Float(n)) => *self = List::Float(vec![*n]),
            (List::Boxed(_), Value::Bool(b)) => *self = List::Bool(vec![*b as u8]),
            (List::Boxed(v), _) => v.push(x),
            _ => self.boxed_mut().push(x),
        }
    }

    pub(crate) fn pop(&mut self) -> Option<Value> {
        match self {
            List::Boxed(v) => v.pop(),
            List::Bool(v) => v.pop().map(|b| Value::Bool(b != 0)),
            List::Int(v) => v.pop().map(Value::Int),
            List::Float(v) => v.pop().map(Value::Float),
        }
    }

    pub(crate) fn clear(&mut self) {
        *self = List::Boxed(Vec::new());
    }

    pub(crate) fn swap(&mut self, a: usize, b: usize) {
        match self {
            List::Boxed(v) => v.swap(a, b),
            List::Bool(v) => v.swap(a, b),
            List::Int(v) => v.swap(a, b),
            List::Float(v) => v.swap(a, b),
        }
    }

    pub(crate) fn reverse(&mut self) {
        match self {
            List::Boxed(v) => v.reverse(),
            List::Bool(v) => v.reverse(),
            List::Int(v) => v.reverse(),
            List::Float(v) => v.reverse(),
        }
    }

    /// The elements `lo..hi` (in range) as a new list of the same kind.
    pub(crate) fn range(&self, lo: usize, hi: usize) -> List {
        match self {
            List::Boxed(v) => List::Boxed(v[lo..hi].to_vec()),
            List::Bool(v) => List::Bool(v[lo..hi].to_vec()),
            List::Int(v) => List::Int(v[lo..hi].to_vec()),
            List::Float(v) => List::Float(v[lo..hi].to_vec()),
        }
    }
}

impl List {
    /// Append the elements of `other`; a packed list takes a packed one of its kind without boxing.
    pub(crate) fn extend_from(&mut self, other: &List) {
        match (&mut *self, other) {
            (List::Int(a), List::Int(b)) => a.extend_from_slice(b),
            (List::Float(a), List::Float(b)) => a.extend_from_slice(b),
            (List::Bool(a), List::Bool(b)) => a.extend_from_slice(b),
            (List::Boxed(a), List::Boxed(b)) => a.extend(b.iter().cloned()),
            (me, other) if me.is_empty() => *me = other.clone(),
            (me, other) => me.boxed_mut().extend(other.iter()),
        }
    }

    pub(crate) fn insert(&mut self, i: usize, x: Value) {
        if self.is_empty() {
            self.push(x);
            return;
        }
        match (&mut *self, &x) {
            (List::Int(v), Value::Int(n)) => v.insert(i, *n),
            (List::Float(v), Value::Float(n)) => v.insert(i, *n),
            (List::Bool(v), Value::Bool(b)) => v.insert(i, *b as u8),
            _ => self.boxed_mut().insert(i, x),
        }
    }

    pub(crate) fn remove(&mut self, i: usize) -> Value {
        match self {
            List::Boxed(v) => v.remove(i),
            List::Bool(v) => Value::Bool(v.remove(i) != 0),
            List::Int(v) => Value::Int(v.remove(i)),
            List::Float(v) => Value::Float(v.remove(i)),
        }
    }
}
