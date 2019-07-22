use core::mem;
use core::marker::PhantomData;

use super::Error;

/// Proof of a single, contiguous allocation for a certain lifetime.
pub struct AllocationProof<'a> {
    begin: usize,
    end: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a> AllocationProof<'a> {
    pub fn new<T>(slice: &'a [T]) -> Self {
        let begin = slice.as_ptr() as usize;
        let end = begin + slice.len() * mem::size_of::<T>();
        AllocationProof {
            begin,
            end,
            phantom: PhantomData,
        }
    }

    /// Construct an allocation proof from a mutable slice.
    ///
    /// The slice is returned in full, it is simply required to be semantically borrowed to the
    /// proof. Contrary to the non-mutable case we can not have a copy of the reference with same
    /// lifetime.
    pub fn new_mut<T>(slice: &'a mut [T])
        -> (Self, &'a mut [T])
    {
        let begin = slice.as_ptr() as usize;
        let end = begin + slice.len() * mem::size_of::<T>();
        (AllocationProof {
            begin,
            end,
            phantom: PhantomData,
        }, slice)
    }


    /// Concatenate two slices within the allocation.
    ///
    /// # Errors
    /// This method returns an `NotAdjacent` error when the slices are outside the allocation or
    /// when the slices are within the allocation but not adjancent.
    pub fn concat_slice<'b: 'a, T>(&self, a: &'b [T], b: &'b [T])
        -> Result<&'b [T], Error>
    {
        if !self.within(a) || !self.within(b) {
            return Err(Error::NotAdjacent)
        }
        
        unsafe {
            // SAFETY both are within the same allocation: this one.
            super::concat_slice(a, b)
        }
    }

    /// Concatenate two strings within the allocation.
    ///
    /// # Errors
    /// This method returns an `NotAdjacent` error when the slices are outside the allocation or
    /// when the slices are within the allocation but not adjancent.
    pub fn concat<'b: 'a>(&self, a: &'b str, b: &'b str)
        -> Result<&'b str, Error>
    {
        if !self.within(a) || !self.within(b) {
            return Err(Error::NotAdjacent)
        }
        
        unsafe {
            // SAFETY both are within the same allocation: this one.
            super::concat(a, b)
        }
    }

    fn within<T: ?Sized>(&self, a: *const T) -> bool {
        let a = a as *const u8 as usize;
        self.begin <= a && self.end >= a
    }
}
