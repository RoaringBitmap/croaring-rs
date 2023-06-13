use crate::Bitset;

/// Iterator over set bits in a bitset
pub struct BitsetIterator<'a> {
    bitset: &'a Bitset,
    current: usize,
}

impl<'a> BitsetIterator<'a> {
    fn next_set_bits(&mut self, buffer: &mut [usize]) -> usize {
        let mut current = self.current;
        let len = unsafe {
            ffi::bitset_next_set_bits(
                &self.bitset.bitset,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut current,
            )
        };
        self.current = current + 1;
        len
    }
}

impl<'a> Iterator for BitsetIterator<'a> {
    type Item = usize;

    #[doc(alias = "bitset_next_set_bit")]
    fn next(&mut self) -> Option<Self::Item> {
        let has_value = unsafe { ffi::bitset_next_set_bit(&self.bitset.bitset, &mut self.current) };
        let value = self.current;
        self.current += 1;
        has_value.then_some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.bitset.size_in_bits() - self.current))
    }

    #[doc(alias = "bitset_next_set_bits")]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;
        let mut buffer = [0; 512];
        loop {
            let count = self.next_set_bits(&mut buffer);
            if count == 0 {
                return acc;
            }
            for &value in &buffer[..count] {
                acc = f(acc, value);
            }
        }
    }
}

impl<'a> IntoIterator for &'a Bitset {
    type Item = usize;
    type IntoIter = BitsetIterator<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Bitset {
    /// Returns an iterator over the set bits in the bitset
    #[inline]
    pub const fn iter(&self) -> BitsetIterator<'_> {
        BitsetIterator {
            bitset: self,
            current: 0,
        }
    }
}
