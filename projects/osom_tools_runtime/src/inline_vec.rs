#![allow(
    clippy::needless_borrow,
    clippy::uninit_assumed_init,
    clippy::cast_possible_truncation
)]

use core::mem::{ManuallyDrop, MaybeUninit, forget};
use core::ops::Deref;

union InlineVecUnion<T, const N: usize> {
    stack_data: ManuallyDrop<[T; N]>,
    heap_data: ManuallyDrop<Box<[T]>>,
}

/// A structure similar to `vec` but holds `N` items inlined.
#[must_use]
pub struct InlineVec<T, const N: usize> {
    data: InlineVecUnion<T, N>,
    len: u32,
    capacity: u32,
}

impl<T, const N: usize> InlineVec<T, N> {
    const _VALIDATE: () = const {
        assert!(
            N > 0,
            "N must be greater than 0. InlineVec with N == 0 is just Vec. Use Vec instead."
        );
        // Note: 2147482623 is (i32::MAX - 1024). This is definitely way too much,
        // but we reserve some space, just in case.
        assert!(
            N < 2147482623,
            "N must be at most 2147482623. Which likely already is waaaay too much."
        );
    };

    /// Pushes a value to the end of the [`InlineVec`].
    ///
    /// Note that the [`InlineVec`] data will be moved to the heap
    /// only when length exceeds `N`. It won't come back from the
    /// heap though.
    pub fn push(&mut self, value: T) {
        unsafe {
            if self.len < N as u32 {
                let data = &mut self.data.stack_data;
                data.as_mut_ptr().add(self.len()).write(value);
                self.len += 1;
                return;
            }

            if self.capacity == N as u32 {
                let new_capacity = self.capacity * 2;
                let mut vec = Vec::<T>::with_capacity(new_capacity as usize);
                vec.resize_with(new_capacity as usize, || MaybeUninit::<T>::uninit().assume_init());
                let stack_data = &self.data.stack_data;
                vec.as_mut_ptr()
                    .copy_from_nonoverlapping(stack_data.as_ptr(), self.len());
                let mut boxed = ManuallyDrop::new(vec.into_boxed_slice());
                boxed.as_mut_ptr().add(self.len()).write(value);
                self.len += 1;
                self.data = InlineVecUnion { heap_data: boxed };
                self.capacity = new_capacity;
                return;
            }

            if self.len == self.capacity {
                let new_capacity = self.capacity * 2;
                let mut vec = Vec::<T>::with_capacity(new_capacity as usize);
                let current_box = ManuallyDrop::take(&mut self.data.heap_data);
                vec.resize_with(new_capacity as usize, || MaybeUninit::<T>::uninit().assume_init());
                vec.as_mut_ptr()
                    .copy_from_nonoverlapping(current_box.as_ptr(), self.len());
                forget(current_box);
                let boxed = ManuallyDrop::new(vec.into_boxed_slice());
                self.data = InlineVecUnion { heap_data: boxed };
                self.capacity = new_capacity;
            }

            let boxed = &mut self.data.heap_data;
            boxed.as_mut_ptr().add(self.len()).write(value);
            self.len += 1;
        }
    }

    /// Creates a new empty [`InlineVec`].
    #[inline]
    pub fn new() -> Self {
        let uninit = unsafe { MaybeUninit::<[T; N]>::uninit().assume_init() };
        Self {
            data: InlineVecUnion {
                stack_data: ManuallyDrop::new(uninit),
            },
            len: 0,
            capacity: N as u32,
        }
    }

    /// Returns the number of elements in the [`InlineVec`].
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the capacity of the [`InlineVec`]. Note that
    /// this is always at least `N`.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Represents current [`InlineVec`] as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let ptr = if self.capacity == N as u32 {
                self.data.stack_data.as_ptr()
            } else {
                self.data.heap_data.as_ptr()
            };

            std::slice::from_raw_parts(ptr, self.len())
        }
    }
}

impl<T, const N: usize> Drop for InlineVec<T, N> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = if self.capacity == N as u32 {
                (&mut self.data.stack_data).as_mut_ptr()
            } else {
                (&mut self.data.heap_data).as_mut_ptr()
            };

            let mut idx = 0;
            while idx < self.len() {
                drop(ptr.read());
                ptr = ptr.add(1);
                idx += 1;
            }
        }
    }
}

impl<T, const N: usize> Default for InlineVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const N: usize> Clone for InlineVec<T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        let slice = self.as_slice();
        for item in slice {
            new.push(item.clone());
        }
        new
    }
}

impl<T, const N: usize> std::fmt::Debug for InlineVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineVec")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}

impl<T: PartialEq, const N: usize, const M: usize> PartialEq<InlineVec<T, M>> for InlineVec<T, N> {
    fn eq(&self, other: &InlineVec<T, M>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for InlineVec<T, N> {}

impl<T, const N: usize> Deref for InlineVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
