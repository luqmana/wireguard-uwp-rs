//! Utilities and helper types that don't quite fit anywhere else.

use std::sync::atomic::{AtomicU32, Ordering};

use windows::{
    self as Windows,
    core::*,
    Win32::Foundation::E_BOUNDS,
    Foundation::Collections::{IIterable, IIterator, IVectorView},
};

/// A simple wrapper around `Vec` which implements the `IVectorView` and
/// `IIterable` interfaces.
#[implement(
    Windows::Foundation::Collections::IIterable<T>,
    Windows::Foundation::Collections::IVectorView<T>
)]
pub struct Vector<T: RuntimeType + 'static>(Vec<T::DefaultType>);

impl<T: RuntimeType + 'static> Vector<T> {
    pub fn new(v: Vec<T::DefaultType>) -> Vector<T> {
        Vector(v)
    }

    fn First(&self) -> Result<IIterator<T>> {
        Ok(VectorIterator::<T> {
            it: self.cast()?,
            curr: AtomicU32::new(0),
        }
        .into())
    }

    fn GetAt(&self, index: u32) -> Result<T> {
        self
            .0
            .get(index as usize)
            // SAFETY: `DefaultType` is a super trait of `RuntimeType`.
            .map(|el| unsafe { DefaultType::from_default(el) })
            .transpose()?
            .ok_or(Error::from(E_BOUNDS))
    }

    fn Size(&self) -> Result<u32> {
        u32::try_from(self.0.len()).map_err(|_| Error::from(E_BOUNDS))
    }

    fn IndexOf(&self, value: &T::DefaultType, index: &mut u32) -> Result<bool> {
        if let Some(idx) = self.0.iter().position(|el| el == value) {
            *index = u32::try_from(idx).map_err(|_| Error::from(E_BOUNDS))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn GetMany(&self, start: u32, items: &mut [T::DefaultType]) -> Result<u32> {
        let sz = u32::try_from(self.0.len()).map_err(|_| Error::from(E_BOUNDS))?;

        if start  >= sz {
            return Err(Error::from(E_BOUNDS));
        }

        let mut count = 0;
        for (item, el) in items.into_iter().zip(self.0[start as usize..].iter()) {
            *item = el.clone();
            count += 1;
        }
        Ok(count)
    }
}

impl<'a, T: RuntimeType + 'static> IntoParam<'a, IVectorView<T>> for Vector<T> {
    fn into_param(self) -> Param<'a, IVectorView<T>> {
        Param::Owned(self.into())
    }
}

/// `IIterator` wrapper for `Vector`
#[implement(Windows::Foundation::Collections::IIterator<T>)]
struct VectorIterator<T: RuntimeType + 'static> {
    /// The underlying object we're iteratoring over
    it: IIterable<T>,
    /// The current position of the iterator
    curr: AtomicU32,
}

impl<T: RuntimeType + 'static> VectorIterator<T> {
    fn Current(&self) -> Result<T> {
        // SAFETY: We know this must be our `Vector` type
        let vec = unsafe { Vector::to_impl(&self.it) };
        vec.GetAt(self.curr.load(Ordering::Relaxed))
    }

    fn HasCurrent(&self) -> Result<bool> {
        // SAFETY: We know this must be our `Vector` type
        let vec = unsafe { Vector::to_impl(&self.it) };
        Ok(vec.0.len() > self.curr.load(Ordering::Relaxed) as usize)
    }

    fn MoveNext(&self) -> Result<bool> {
        // SAFETY: We know this must be our `Vector` type
        let vec = unsafe { Vector::to_impl(&self.it) };
        let old = self.curr.fetch_add(1, Ordering::Relaxed) as usize;
        Ok(vec.0.len() > old + 1)
    }

    fn GetMany(&self, items: &mut [T::DefaultType]) -> Result<u32> {
        // SAFETY: We know this must be our `Vector` type
        let vec = unsafe { Vector::to_impl(&self.it) };
        vec.GetMany(0, items)
    }
}