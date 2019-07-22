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
    /// Construct an allocation proof from a non-mutable value.
    ///
    /// The instance owns a borrow on the object that was passed in for the duration of the
    /// lifetime `'a`. It must thus not be modified or deallocated. Since all objects live within a
    /// single allocation, all references pointing into that object's memory share the same
    /// undlerying allocation.
    pub fn new<T: ?Sized>(obj: &'a T) -> Self {
        let begin = obj as *const T as *const u8 as usize;
        let end = begin + mem::size_of_val(obj);
        AllocationProof {
            begin,
            end,
            phantom: PhantomData,
        }
    }

    /// Construct an allocation proof from a mutable value.
    ///
    /// The value is returned in full, it is simply required to be semantically borrowed to the
    /// proof. Contrary to the non-mutable case we can not have a copy of the reference with same
    /// lifetime.
    pub fn new_mut<T: ?Sized>(obj: &'a mut T)
        -> (Self, &'a mut T)
    {
        let begin = obj as *mut T as *const u8 as usize;
        let end = begin + mem::size_of_val(obj);
        (AllocationProof {
            begin,
            end,
            phantom: PhantomData,
        }, obj)
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

    /// Concatenate two slices within the allocation without checking their order.
    ///
    /// # Errors
    /// This method returns an `NotAdjacent` error when the slices are outside the allocation or
    /// when the slices are within the allocation but not adjancent.
    pub fn concat_slice_unordered<'b: 'a, T>(&self, a: &'b [T], b: &'b [T])
        -> Result<&'b [T], Error>
    {
        if !self.within(a) || !self.within(b) {
            return Err(Error::NotAdjacent)
        }
        
        unsafe {
            // SAFETY both are within the same allocation: this one.
            super::concat_slice_unordered(a, b)
        }
    }

    /// Concatenate two strings within the allocation without checking their order.
    ///
    /// # Errors
    /// This method returns an `NotAdjacent` error when the slices are outside the allocation or
    /// when the slices are within the allocation but not adjancent.
    pub fn concat_unordered<'b: 'a>(&self, a: &'b str, b: &'b str)
        -> Result<&'b str, Error>
    {
        if !self.within(a) || !self.within(b) {
            return Err(Error::NotAdjacent)
        }
        
        unsafe {
            // SAFETY both are within the same allocation: this one.
            super::concat_unordered(a, b)
        }
    }

    fn within<T: ?Sized>(&self, a: &T) -> bool {
        let a_len = mem::size_of_val(a);
        let a = a as *const T as *const u8 as usize;
        let a_end = a + a_len;
        self.begin <= a && a_end <= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::{AllocationProof, Error};

    #[test]
    fn simple_success() {
        let s = "0123456789";
        let proof = AllocationProof::new(s);
        assert_eq!(Ok("0123456"), proof.concat(&s[..5], &s[5..7]));
        assert_eq!(Ok("0123456"), proof.concat_unordered(&s[..5], &s[5..7]));
    }

    #[test]
    fn unordered() {
        let s = "0123456789";
        let proof = AllocationProof::new(s);
        assert_eq!(Err(Error::NotAdjacent), proof.concat(&s[5..7], &s[..5]));
        assert_eq!(Ok("0123456"), proof.concat_unordered(&s[5..7], &s[..5]));
    }

    #[test]
    fn simple_fail() {
        let s = "0123456789";
        let proof = AllocationProof::new(s);
        assert_eq!(Err(Error::NotAdjacent), proof.concat(&s[..5], &s[6..7]))
    }

    #[test]
    fn other_alloc_fail() {
        let xa = [0; 8];
        let s = [0; 8];
        let xb = [0; 8];

        let proof = AllocationProof::new(&s);
        assert_eq!(Err(Error::NotAdjacent), proof.concat_slice_unordered(&xa[..], &s[..]));
        assert_eq!(Err(Error::NotAdjacent), proof.concat_slice_unordered(&s[..], &xb[..]));

        assert_eq!(Err(Error::NotAdjacent), proof.concat_slice_unordered(&xa[..2], &xa[2..]));
        assert_eq!(Err(Error::NotAdjacent), proof.concat_slice_unordered(&xa[..2], &xb[2..]));
    }

    #[test]
    fn empty_str() {
        let s = "0123";
        let proof = AllocationProof::new(s);
        assert_eq!(Ok("0123"), proof.concat(&s[..0], s));
        assert_eq!(Ok("0123"), proof.concat_unordered(&s[..0], s));
        assert_eq!(Ok("0123"), proof.concat_unordered(s, &s[..0]));
        assert_eq!(Ok("0123"), proof.concat(s, &s[4..]));
        assert_eq!(Ok("0123"), proof.concat_unordered(s, &s[4..]));
        assert_eq!(Ok("0123"), proof.concat_unordered(&s[4..], s));
    }
}
