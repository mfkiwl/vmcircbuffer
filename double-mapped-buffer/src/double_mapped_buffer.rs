use std::marker::PhantomData;
use std::mem;
use std::slice;

use super::DoubleMappedBufferError;
use super::DoubleMappedBufferImpl;

pub struct DoubleMappedBuffer<T> {
    buffer: DoubleMappedBufferImpl,
    _p: PhantomData<T>,
}

impl<T> DoubleMappedBuffer<T> {
    pub fn new(min_items: usize) -> Result<Self, DoubleMappedBufferError> {
        match DoubleMappedBufferImpl::new(min_items, mem::size_of::<T>(), mem::align_of::<T>()) {
            Ok(buffer) => Ok(DoubleMappedBuffer {
                buffer,
                _p: PhantomData,
            }),
            Err(e) => Err(e),
        }
    }

    /// # Safety
    pub unsafe fn slice(&self) -> &[T] {
        let addr = self.buffer.addr() as usize;
        debug_assert_eq!(addr % mem::align_of::<T>(), 0);
        slice::from_raw_parts(addr as *const T, self.buffer.len())
    }

    /// # Safety
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn slice_mut(&self) -> &mut [T] {
        let addr = self.buffer.addr() as usize;
        debug_assert_eq!(addr % mem::align_of::<T>(), 0);
        slice::from_raw_parts_mut(addr as *mut T, self.buffer.len())
    }

    /// # Safety
    pub unsafe fn slice_with_offset(&self, offset: usize) -> &[T] {
        let addr = self.buffer.addr() as usize;
        debug_assert_eq!(addr % mem::align_of::<T>(), 0);
        debug_assert!(offset <= self.buffer.len());
        slice::from_raw_parts((addr as *const T).add(offset), self.buffer.len())
    }

    /// # Safety
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn slice_mut_with_offset(&self, offset: usize) -> &mut [T] {
        let addr = self.buffer.addr() as usize;
        debug_assert_eq!(addr % mem::align_of::<T>(), 0);
        debug_assert!(offset <= self.buffer.len());
        slice::from_raw_parts_mut((addr as *mut T).add(offset), self.buffer.len())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pagesize;
    use std::mem;
    use std::sync::atomic::compiler_fence;
    use std::sync::atomic::Ordering;

    #[test]
    fn byte_buffer() {
        let b = DoubleMappedBuffer::<u8>::new(123).expect("failed to create buffer");
        let ps = pagesize();

        assert_eq!(b.len() * mem::size_of::<u8>() % ps, 0);
        assert_eq!(b.buffer.addr() % mem::align_of::<u8>(), 0);

        unsafe {
            let s = b.slice_mut();
            assert_eq!(s.len(), b.len());
            assert_eq!(s.as_mut_ptr() as usize, b.buffer.addr());

            for (i, v) in s.iter_mut().enumerate() {
                *v = (i % 128) as u8;
            }

            compiler_fence(Ordering::SeqCst);

            let s = b.slice_with_offset(b.len());
            assert_eq!(
                s.as_ptr() as usize,
                b.buffer.addr() + b.len() * mem::size_of::<u8>()
            );
            for (i, v) in s.iter().enumerate() {
                assert_eq!(*v, (i % 128) as u8);
            }

            compiler_fence(Ordering::SeqCst);
            b.slice_mut()[0] = 123;
            compiler_fence(Ordering::SeqCst);
            assert_eq!(b.slice_with_offset(b.len())[0], 123);
        }
    }

    #[test]
    fn u32_buffer() {
        let b = DoubleMappedBuffer::<u32>::new(12311).expect("failed to create buffer");
        let ps = pagesize();

        assert_eq!(b.len() * mem::size_of::<u32>() % ps, 0);
        assert_eq!(b.buffer.addr() % mem::align_of::<u32>(), 0);

        unsafe {
            let s = b.slice_mut();
            assert_eq!(s.len(), b.len());
            assert_eq!(s.as_mut_ptr() as usize, b.buffer.addr());

            for (i, v) in s.iter_mut().enumerate() {
                *v = (i % 128) as u32;
            }

            compiler_fence(Ordering::SeqCst);

            let s = b.slice_with_offset(b.len());
            assert_eq!(
                s.as_ptr() as usize,
                b.buffer.addr() + b.len() * mem::size_of::<u32>()
            );
            for (i, v) in s.iter().enumerate() {
                assert_eq!(*v, (i % 128) as u32);
            }

            compiler_fence(Ordering::SeqCst);
            b.slice_mut()[0] = 123;
            compiler_fence(Ordering::SeqCst);
            assert_eq!(b.slice_with_offset(b.len())[0], 123);
        }
    }

    #[test]
    fn many_buffers() {
        let _b0 = DoubleMappedBuffer::<u32>::new(123).expect("failed to create buffer");
        let _b1 = DoubleMappedBuffer::<u32>::new(456).expect("failed to create buffer");

        let mut v = Vec::new();

        for _ in 0..100 {
            v.push(DoubleMappedBuffer::<u32>::new(123).expect("failed to create buffer"));
        }
    }
}
