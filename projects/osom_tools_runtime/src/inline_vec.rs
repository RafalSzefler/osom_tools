#![allow(
    clippy::needless_borrow,
    clippy::uninit_assumed_init,
    clippy::cast_possible_truncation
)]

use core::mem::ManuallyDrop;
use core::ops::Deref;
use std::alloc::Layout;
use std::ptr::dangling_mut;

union InlineVecUnion<T, const N: usize> {
    stack_data: ManuallyDrop<[T; N]>,
    heap_data: *mut T,
}

/// A structure similar to `vec` but holds `N` items inlined.
#[must_use]
pub struct InlineVec<T, const N: usize> {
    data: InlineVecUnion<T, N>,
    len: u32,
    capacity: u32,
}

impl<T, const N: usize> InlineVec<T, N> {
    const MAX_SIZE: usize = (i32::MAX - 1024) as usize;

    const _VALIDATE: () = const {
        assert!(
            N > 0,
            "N must be greater than 0. InlineVec with N == 0 is just Vec. Use Vec instead."
        );
        // Note: 2147482623 is (i32::MAX - 1024). This is definitely way too much,
        // but we reserve some space, just in case.
        assert!(
            N < Self::MAX_SIZE,
            "N must be at most 2147482623. Which likely already is waaaay too much."
        );
    };

    #[inline(always)]
    const fn layout(size: usize) -> Layout {
        let real_size = size * size_of::<T>();
        let alignment = align_of::<T>();
        unsafe { Layout::from_size_align_unchecked(real_size, alignment) }
    }

    #[inline(always)]
    fn allocate_memory(size: usize) -> *mut T {
        let new_memory = unsafe { std::alloc::alloc(Self::layout(size)) };
        assert!(
            !new_memory.is_null(),
            "Couldn't allocate new memory for InlineVec resize."
        );
        let result = new_memory.cast::<T>();
        assert!(result.is_aligned(), "Newly allocated memory is not aligned correctly.");
        result
    }

    /// Pushes a value to the end of the [`InlineVec`].
    ///
    /// Note that the [`InlineVec`] data will be moved to the heap
    /// only when length exceeds `N`. It won't come back from the
    /// heap though.
    ///
    /// # Panics
    ///
    /// Only during reallocation, when memory limits are exceeded, or
    /// memory allocation is not possible for whatever reason.
    pub fn push(&mut self, value: T) {
        unsafe {
            if self.capacity == N as u32 {
                if self.len < N as u32 {
                    let data = &mut self.data.stack_data;
                    data.as_mut_ptr().add(self.len()).write(value);
                    self.len += 1;
                    return;
                }
                let new_capacity = (self.capacity * 2) as usize;
                assert!(new_capacity <= Self::MAX_SIZE, "New capacity exceeded 2147482623.");
                let new_memory = Self::allocate_memory(new_capacity);
                let stack_data = &self.data.stack_data;
                new_memory.copy_from_nonoverlapping(stack_data.as_ptr(), self.len());
                new_memory.add(self.len()).write(value);
                self.len += 1;
                self.data = InlineVecUnion { heap_data: new_memory };
                self.capacity = new_capacity as u32;
                return;
            }

            if self.len == self.capacity {
                let new_capacity = (self.capacity * 2) as usize;
                assert!(new_capacity <= Self::MAX_SIZE, "New capacity exceeded 2147482623.");
                let new_memory = Self::allocate_memory(new_capacity);
                let old_memory = self.data.heap_data;
                new_memory.copy_from_nonoverlapping(old_memory, self.len());
                let old_layout = Self::layout(self.capacity());
                std::alloc::dealloc(old_memory.cast(), old_layout);
                self.data.heap_data = new_memory;
                self.capacity = new_capacity as u32;
            }

            self.data.heap_data.add(self.len()).write(value);
            self.len += 1;
        }
    }

    /// Pops last element from the [`InlineVec`],
    /// decreasing its size.
    ///
    /// # Returns
    ///
    /// * `Some(T)` if `self.len() > 0`
    /// * `None` otherwise
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.pop_unchecked() })
        }
    }

    /// Unsafe variant of [`pop`][`Self::pop`].
    ///
    /// # Safety
    ///
    /// Returns `T` if `self.len() > 0` and decrease the
    /// [`InlineVec`] size. The behaviour is undefined if
    /// `self.len() == 0`.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty(), "Tried pop_unchecked on length 0 InlineVec.");
        unsafe {
            let ptr = if self.capacity == N as u32 {
                self.data.stack_data.as_ptr()
            } else {
                self.data.heap_data
            };
            self.len -= 1;
            ptr.add(self.len as usize).read()
        }
    }

    /// Creates a new empty [`InlineVec`].
    #[inline]
    pub fn new() -> Self {
        Self {
            data: InlineVecUnion {
                heap_data: dangling_mut(),
            },
            len: 0,
            capacity: N as u32,
        }
    }

    /// Returns the number of elements in the [`InlineVec`].
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the [`InlineVec`] is empty,
    /// otherwise `false`.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the [`InlineVec`]. Note that
    /// this is always at least `N`.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Represents current [`InlineVec`] as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let ptr = if self.capacity == N as u32 {
                self.data.stack_data.as_ptr()
            } else {
                self.data.heap_data
            };

            std::slice::from_raw_parts(ptr, self.len())
        }
    }
}

impl<T, const N: usize> Drop for InlineVec<T, N> {
    fn drop(&mut self) {
        unsafe {
            if core::mem::needs_drop::<T>() {
                let mut ptr = if self.capacity == N as u32 {
                    (&mut self.data.stack_data).as_mut_ptr()
                } else {
                    self.data.heap_data
                };

                let mut idx = 0;
                while idx < self.len() {
                    drop(ptr.read());
                    ptr = ptr.add(1);
                    idx += 1;
                }
            }

            if self.capacity > N as u32 {
                let layout = Self::layout(self.capacity());
                std::alloc::dealloc(self.data.heap_data.cast(), layout);
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
            .field("N", &N)
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
