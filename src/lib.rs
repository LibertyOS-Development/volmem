#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(const_generics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", allow(incomplete_features))]
#![cfg_attr(all(feature = "unstable", test), feature(slice_as_chunks))]

use access::{ReadOnly, ReadWrite, Readable, Writable, WriteOnly};
use core::{
	fmt,
	marker::PhantomData,
	ops::Deref,
	ops::{DerefMut, Index, IndexMut},
	ptr,
	slice::SliceIndex,
};
#[cfg(feature = "unstable")]
use core::{
	intrinsics,
	ops::{Range, RangeBounds},
	slice::range,
};

pub mod access;

#[derive(Clone)]
#[repr(transparent)]
pub struct Volmem<R, A = ReadWrite>
{
	refer: R,
	acc: PhantomData<A>,
}

impl<R> Volmem<R>
{
	pub const fn new(refer: R) -> Volmem<R>
	{
		Volmem
		{
			refer,
			acc: PhantomData,
		}
	}
	pub const fn new_rdonly(refer: R) -> Volmem<R, ReadOnly>
	{
		Volmem
		{
			refer,
			acc: PhantomData,
		}
	}
	pub const fn new_wronly(refer: R) -> Volmem<R, WriteOnly>
	{
		Volmem
		{
			refer,
			acc: PhantomData,
		}
	}
}

impl<R, T, A> Volmem<R, A>
where
	R: Deref<Target = T>,
	T: Copy,
{
	pub fn read(&self) -> T
	where
		A: Readable,
	{
		unsafe
		{
			ptr::read_volatile(&*self.refer)
		}
	}
	pub fn write(&mut self, value: T)
	where
		A: Writable,
		R: DerefMut,
	{
		unsafe
		{
			ptr::write_volatile(&mut *self.refer, value)
		};
	}
	pub fn update<F>(&mut self, f: F)
	where
		A: Readable + Writable,
		R: DerefMut,
		F: FnOnce(&mut T),
	{
		let mut value = self.read();
		f(&mut value);
		self.write(value);
	}
}

impl<R, A> Volmem<R, A>
{
	pub fn extractinner(self) -> R
	{
		self.refer
	}
}

impl<R, T, A> Volmem<R, A>
where
	R: Deref<Target = T>,
	T: ?Sized,
{
	pub fn map<'a, F, U>(&'a self, f: F) -> Volmem<&'a U, A>
	where
		F: FnOnce(&'a T) -> &'a U,
		U: ?Sized,
		T: 'a,
	{
		Volmem
		{
			refer: f(self.refer.deref()),
			acc: self.acc,
		}
	}
	pub fn map_mut<'a, F, U>(&'a mut self, f: F) -> Volmem<&'a mut U, A>
	where
		F: FnOnce(&mut T) -> &mut U,
		R: DerefMut,
		U: ?Sized,
		T: 'a,
	{
		Volmem
		{
			refer: f(&mut self.refer),
			acc: self.acc,
		}
	}
}

impl<T, R, A> Volmem<R, A>
where
	R: Deref<Target = [T]>,
{
	pub fn idx<'a, I>(&'a self, idx: I) -> Volmem<&'a I::Output, A>
	where
		I: SliceIndex<[T]>,
		T: 'a,
	{
		self.map(|slice| slice.index(idx))
	}
	pub fn idxmut<'a, I>(&'a mut self, idx: I) -> Volmem<&mut I::Output, A>
	where
		I: SliceIndex<[T]>,
		R: DerefMut,
		T: 'a,
	{
		self.map_mut(|slice| slice.index_mut(idx))
	}

	#[cfg(feature = "unstable")]
	pub fn copy_into_slice(&self, dst: &mut [T])
	where
		T: Copy,
	{
		assert_eq!(self.refer.len(), dst.len(), "[ERR] DEST AND SRC SLICES HAVE DIFFERENT LENGTHS");
		unsafe
		{
			intrinsics::volatile_copy_nonoverlapping_memory(dst.as_mut_ptr(), self.refer.as_ptr(), self.refer.len());
		}
	}
	#[cfg(feature = "unstable")]
	pub fn copy_from_slice(&mut self, src: &[T])
	where
		T: Copy,
		R: DerefMut,
	{
		assert_eq!(self.refer.len(), src.len(), "[ERR] DEST AND SRC SLICES HAVE DIFFERENT LENGTHS");
		unsafe
		{
			intrinsics::volatile_copy_nonoverlapping_memory(self.refer.as_mut_ptr(), src.as_ptr(), self.refer.len());
		}
	}
	#[cfg(feature = "unstable")]
	pub fn copy_within(&mut self, src: impl RangeBounds<usize>, dest: usize)
	where
		T: Copy,
		R: DerefMut,
	{
		let Range
		{
			start: src_start,
			end: src_end,
		} = range(src, ..self.refer.len());
		let count = src_end - src_start;
		assert!(dest <= self.refer.len() - count, "[ERR] DEST IS OUT OF BOUNDS");
		unsafe
		{
			intrinsics::volatile_copy_memory(self.refer.as_mut_ptr().add(dest), self.refer.as_ptr().add(src.start), count);
		}
	}
}

impl<R, A> Volmem<R, A>
where
	R: Deref<Target = [u8]>,
{
	#[cfg(feature = "unstable")]
	pub fn fill(&mut self, value: u8)
	where
		R: DerefMut,
	{
		unsafe
		{
			intrinsics::volatile_set_memory(self.refer.as_mut_ptr(), value, self.refer.len());
		}
	}
}

#[cfg(feature = "unstable")]
impl<R, A, T, const N: usize> Volmem<R, A>
where
	R: Deref<Target = [T; N]>,
{
	pub fn as_slice(&self) -> Volmem<&[T], A>
	{
		self.map(|array| &array[..])
	}
	pub fn as_mut_slice(&mut self) -> Volmem<&mut [T], A>
	where
		R: DerefMut,
	{
		self.map_mut(|array| &mut array[..])
	}
}

impl<R> Volmem<R>
{
	pub fn readonly(self) -> Volmem<R, ReadOnly>
	{
		Volmem
		{
			refer: self.refer,
			acc: PhantomData,
		}
	}
	pub fn writeonly(self) -> Volmem<R, WriteOnly>
	{
		Volmem
		{
			refer: self.refer,
			acc: PhantomData,
		}
	}
}

impl<R, T, A> fmt::Debug for Volmem<R, A>
where
	R: Deref<Target = T>,
	T: Copy + fmt::Debug,
	A: Readable,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		f.debug_tuple("Volmem").field(&self.read()).finish()
	}
}

impl<R> fmt::Debug for Volmem<R, WriteOnly>
where
	R: Deref,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		f.debug_tuple("Volmem").field(&"[writeonly]").finish()
	}
}
