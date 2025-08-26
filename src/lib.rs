use std::{
    alloc::{self, Layout},
    ops::{Index, IndexMut},
    ptr, slice,
};

pub struct RawVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        RawVec {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe { self.ptr.add(self.len).write(value) }
        self.len += 1;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            slice: unsafe { slice::from_raw_parts(self.ptr, self.len) },
            pos: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            ptr: self.ptr,
            end: unsafe { self.ptr.add(self.len) },
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    //non pub method to grow the vec if and when needed
    fn grow(&mut self) {
        let elem_size = size_of::<T>();
        let align = align_of::<T>();
        let new_cap = if self.cap == 0 {
            1
        } else {
            self.cap + self.cap / 2
        };
        let new_cap = new_cap.max(self.len + 1);
        unsafe {
            let layout = Layout::from_size_align_unchecked(new_cap * elem_size, align);
            let new_ptr = if self.cap == 0 {
                alloc::alloc(layout) as *mut T
            } else {
                let old_size = self.cap * elem_size;
                let old_layout = Layout::from_size_align_unchecked(old_size, align);
                alloc::realloc(self.ptr as *mut u8, old_layout, new_cap * elem_size) as *mut T
            };
            if new_ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            self.ptr = new_ptr;
            self.cap = new_cap
        }
    }
}

// -------impl Index and IndexMut for RawVec--------
impl<T> Index<usize> for RawVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            None => {
                panic!("Index out of bounds!")
            }
            Some(x) => x,
        }
    }
}

impl<T> IndexMut<usize> for RawVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            None => {
                panic!("Index out of bounds!")
            }
            Some(x) => x,
        }
    }
}

//------Iterators------
pub struct Iter<'a, T> {
    slice: &'a [T],
    pos: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.slice.len() {
            return None;
        }
        self.pos += 1;
        Some(&self.slice[self.pos - 1])
    }
}

pub struct IterMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            return None;
        }
        unsafe {
            let ptr_clone = self.ptr;
            self.ptr = self.ptr.add(1);
            ptr_clone.as_mut()
        }
    }
}

//-----impl Drop for RawVec-----
impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        unsafe {
            for i in 0..self.len {
                self.ptr.add(i).drop_in_place();
            }
            let layout =
                Layout::from_size_align_unchecked(self.cap * size_of::<T>(), align_of::<T>());
            alloc::dealloc(self.ptr as *mut u8, layout);
        }
    }
}
