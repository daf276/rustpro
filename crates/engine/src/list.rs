use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

// A Vec that keeps up to N items inline: a short list clones as plain bytes, and only a longer one allocates.
#[derive(Debug)]
pub struct List<T: Copy + Default, const N: usize> {
    len: u16,
    inline: [T; N],
    // Holds every item, and `len` stays 0, while not empty. Keeps its capacity once emptied, so a list that spills
    // again does not allocate again; a clone of an empty Vec allocates nothing.
    spilled: Vec<T>,
}

// Not derived, so that the usual empty spill skips Vec::clone: a Duel clone copies dozens of lists.
impl<T: Copy + Default, const N: usize> Clone for List<T, N> {
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            inline: self.inline,
            spilled: if self.spilled.is_empty() {
                Vec::new()
            } else {
                self.spilled.clone()
            },
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.len = source.len;
        self.inline = source.inline;
        self.spilled.clone_from(&source.spilled);
    }
}

impl<T: Copy + Default, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        Self {
            len: 0,
            inline: [T::default(); N],
            spilled: Vec::new(),
        }
    }
}

impl<T: Copy + Default, const N: usize> List<T, N> {
    fn spill(&mut self) -> &mut Vec<T> {
        let len = std::mem::take(&mut self.len);
        if self.spilled.is_empty() {
            self.spilled.reserve(2 * N);
            self.spilled.extend(self.inline.into_iter().take(usize::from(len)));
        }
        &mut self.spilled
    }

    pub fn push(&mut self, item: T) {
        if self.spilled.is_empty()
            && let Some(slot) = self.inline.get_mut(usize::from(self.len))
        {
            *slot = item;
            self.len += 1;
        } else {
            self.spill().push(item);
        }
    }

    pub fn insert(&mut self, index: usize, item: T) {
        let len = usize::from(self.len);
        if self.spilled.is_empty() && len < N && index <= len {
            self.inline.copy_within(index..len, index + 1);
            if let Some(slot) = self.inline.get_mut(index) {
                *slot = item;
            }
            self.len += 1;
        } else {
            self.spill().insert(index, item);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if !self.spilled.is_empty() {
            return (index < self.spilled.len()).then(|| self.spilled.remove(index));
        }
        let len = usize::from(self.len);
        let item = self.get(index).copied()?;
        self.inline.copy_within(index + 1..len, index);
        self.len -= 1;
        Some(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.spilled.is_empty() {
            return self.spilled.pop();
        }
        self.len = self.len.checked_sub(1)?;
        self.inline.get(usize::from(self.len)).copied()
    }

    pub fn resize(&mut self, len: usize, item: T) {
        let current = usize::from(self.len);
        if self.spilled.is_empty() && len <= N {
            if let Some(added) = self.inline.get_mut(current..len) {
                added.fill(item);
            }
            self.len = len as u16;
        } else {
            self.spill().resize(len, item);
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.spilled.clear();
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        if !self.spilled.is_empty() {
            self.spilled.retain(keep);
            return;
        }
        let mut kept = 0;
        for read in 0..usize::from(self.len) {
            if let Some(&item) = self.inline.get(read)
                && keep(&item)
            {
                if let Some(slot) = self.inline.get_mut(kept) {
                    *slot = item;
                }
                kept += 1;
            }
        }
        self.len = kept as u16;
    }

    // Moves every item of `other` to the end of this list.
    pub fn append<const M: usize>(&mut self, other: &mut List<T, M>) {
        self.extend(other.iter().copied());
        other.clear();
    }
}

impl<T: Copy + Default + PartialEq, const N: usize> PartialEq for List<T, N> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Copy + Default, const N: usize> Deref for List<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        if self.spilled.is_empty() {
            self.inline.get(..usize::from(self.len)).unwrap_or_default()
        } else {
            &self.spilled
        }
    }
}

impl<T: Copy + Default, const N: usize> DerefMut for List<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        if self.spilled.is_empty() {
            self.inline.get_mut(..usize::from(self.len)).unwrap_or_default()
        } else {
            &mut self.spilled
        }
    }
}

impl<T: Copy + Default, const N: usize> Extend<T> for List<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        items.into_iter().for_each(|item| self.push(item));
    }
}

impl<T: Copy + Default, const N: usize> FromIterator<T> for List<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut list = Self::default();
        list.extend(items);
        list
    }
}

impl<'a, T: Copy + Default, const N: usize> IntoIterator for &'a List<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter<T: Copy + Default, const N: usize> {
    list: List<T, N>,
    next: usize,
    end: usize,
}

impl<T: Copy + Default, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let items: &[T] = if self.list.spilled.is_empty() {
            &self.list.inline
        } else {
            &self.list.spilled
        };
        let item = items.get(self.next).copied().filter(|_| self.next < self.end);
        self.next += 1;
        item
    }
}

impl<T: Copy + Default, const N: usize> IntoIterator for List<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            end: self.len(),
            list: self,
            next: 0,
        }
    }
}

// A Vec in chunks of N items (a multiple of 8) that clones share until one of them writes to a chunk, which then copies
// that chunk alone; the last chunk's slots past the end stay readable. It also marks the runs of 8 items that changed
// (the last mark covers every item from 504 on), so that a copy kept in sync with it, such as a script snapshot, is
// brought up to date by copying only those into its own chunks.
#[derive(Debug, Clone)]
pub struct Shared<T, const N: usize> {
    chunks: Vec<Arc<[T; N]>>,
    len: usize,
    changed: Cell<u64>,
}

const RUN: usize = 8;

impl<T, const N: usize> Default for Shared<T, N> {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            len: 0,
            changed: Cell::new(0),
        }
    }
}

impl<T: Copy + Default, const N: usize> Shared<T, N> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.chunks.get(index / N)?.get(index % N)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.chunks.iter().flat_map(|chunk| chunk.iter()).take(self.len)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let chunk = self.chunks.get_mut(index / N)?;
        self.changed.set(self.changed.get() | 1 << (index / RUN).min(63));
        Arc::make_mut(chunk).get_mut(index % N)
    }

    pub fn push(&mut self, item: T) {
        if self.len.is_multiple_of(N) {
            self.chunks.push(Arc::new([T::default(); N]));
        }
        self.len += 1;
        if let Some(slot) = self.get_mut(self.len - 1) {
            *slot = item;
        }
    }

    // Makes this equal to `source`. The two must have been equal apart from the runs either marked since, as a copy is
    // to its original after either synced with the other.
    pub fn sync_from(&mut self, source: &Self) {
        self.chunks.truncate(source.chunks.len());
        let common = N * self.chunks.len();
        self.chunks.extend(source.chunks.iter().skip(self.chunks.len()).cloned());
        let mut changed = self.changed.take() | source.changed.take();
        while changed != 0 {
            let mark = changed.trailing_zeros() as usize;
            changed &= changed - 1;
            let end = if mark == 63 { common } else { common.min(RUN * mark + RUN) };
            for start in (RUN * mark..end).step_by(RUN) {
                let (chunk, run) = (start / N, start % N..start % N + RUN);
                if let (Some(to), Some(from)) = (self.chunks.get_mut(chunk), source.chunks.get(chunk))
                    && !Arc::ptr_eq(to, from)
                    && let (Some(to), Some(from)) = (Arc::make_mut(to).get_mut(run.clone()), from.get(run))
                {
                    to.copy_from_slice(from);
                }
            }
        }
        self.len = source.len;
    }

    // The indices of the items in chunks that `other` does not share.
    pub fn unshared<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = usize> + 'a {
        let shared = |chunk: usize, items: &Arc<[T; N]>| other.chunks.get(chunk).is_some_and(|theirs| Arc::ptr_eq(items, theirs));
        (self.chunks.iter().enumerate())
            .filter(move |&(chunk, items)| !shared(chunk, items))
            .flat_map(|(chunk, _)| N * chunk..N * chunk + N)
            .filter(move |&index| index < self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Op {
        Push(u16),
        Insert(usize, u16),
        Remove(usize),
        Pop,
        Clear,
    }

    #[test]
    fn should_copy_only_the_chunks_written() {
        let mut original: Shared<u16, 8> = Shared::default();
        (0..300).for_each(|item| original.push(item));
        let mut copy = original.clone();
        original.push(300);
        *original.get_mut(3).unwrap() = 7;
        assert_eq!(
            original.unshared(&copy).collect::<Vec<_>>(),
            (0..8).chain(296..301).collect::<Vec<_>>()
        );
        assert_eq!(copy.get(3), Some(&3));
        copy.sync_from(&original);
        assert!(copy.len() == 301 && (0..301).all(|index| copy.get(index) == original.get(index)));
    }

    #[test]
    fn should_behave_like_a_vec_inline_and_spilled() {
        let (mut list, mut vec) = (List::<u16, 3>::default(), Vec::new());
        let ops = [
            Op::Push(1),
            Op::Insert(0, 2),
            Op::Insert(1, 3),
            Op::Insert(3, 4),
            Op::Remove(1),
            Op::Pop,
            Op::Clear,
            Op::Push(5),
            Op::Push(6),
            Op::Remove(0),
            Op::Remove(5),
            Op::Pop,
            Op::Pop,
        ];
        for op in ops {
            match op {
                Op::Push(item) => (list.push(item), vec.push(item)).0,
                Op::Insert(index, item) => (list.insert(index, item), vec.insert(index, item)).0,
                Op::Remove(index) => assert_eq!(list.remove(index), (index < vec.len()).then(|| vec.remove(index))),
                Op::Pop => assert_eq!(list.pop(), vec.pop()),
                Op::Clear => (list.clear(), vec.clear()).0,
            }
            assert_eq!(&*list, vec.as_slice());
        }
    }
}
