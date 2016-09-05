/*!
Reuse allocations when converting the elements of a vector, boxed slice or box
to a compatible type.

The `*_in_place()` methods will panic if the types are not compatible,
while the others will fall back to iterating and collecting.

I might add methods to the traits without a default impl or a major version bump;
implement them for other types at your own risk.

# Examples:

```
extern crate map_in_place;
use map_in_place::MapVecInPlace;
fn main() {
    let v = vec![8_u32,29,14,5];
    let v = v.filter_map(|n| if n < 10 {Some( (n as u8+b'0') as char)}
                             else      {None}
                        );// happens in place
    assert_eq!(&v, &['8','5']);
    let v = v.map(|c| c as u8);// falls back to iterators
    assert_eq!(&v[..], &b"85"[..]);
}
```
*/


extern crate scopeguard;
use scopeguard::guard;
use std::{mem, ptr};


// Error messages used by {map,retain}_in_place().
static ERR_ZERO_SIZED: &'static str
     = "The optimization doesn't make sense fore zero-sized types";
static ERR_ALIGNMENT: &'static str = "`A` and `B` have different alignment";
static ERR_NEED_EXACT_SIZE: &'static str = "`A` and `B` have different sizes";
static ERR_NEED_DIVISIBLE_SIZE: &'static str
     = "The size of `A` is not equal to or a multiple of the size of `B`";



pub trait MapBoxInPlace<A> {
    /// Replace the value with one that might have a different type.
    ///
    /// If the types have identical size and alignment
    /// the heap allocation will be reused.
    fn map<B,F:FnOnce(A)->B>(self, f: F) -> Box<B>;

    /// Replace the value in the box with one that might be of another type
    /// as long the new type has identical size and alignment with the old one.
    ///
    /// # Panics:
    /// If the new type doesn't have the same size and alignment.
    fn map_in_place<B,F:FnOnce(A)->B>(self, f: F) -> Box<B>;
}

pub trait MapSliceInPlace<A> {
    /// Map the elements as in an iterator, but reuse the allocation if
    /// the types have identical size and alignment.
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]>;

    /// Map the elements as in an iterator, but reuse the allocation.
    ///
    /// # Panics:
    /// If the old and new types doesn't have identical size and alignment.
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]>;
}

pub trait MapVecInPlace<A> {
    /// Shorter than `.into_iter().map(f).collect::<Vec<_>>()`,
    /// and faster if the types have identical alignment and the size of `A` is
    /// divisible by the size of `B`: Then the allocation is reused.
    ///
    /// This function doesn't attempt to optimize cases
    /// where the size of `A` is a multiple of the size of `B`:
    /// I think `capacity` is rarely twice or more `size`, nevermind 3x or 4x.
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Vec<B>;

    /// Reuse the memory owned by `self` when converting the elements
    /// to a different type.
    /// For this to be safe the types must have identical alignment and
    /// the size of `A` must be divisible by the size of `B`
    /// (`size_of::<A>() % size_of::<B>() == 0`).
    ///
    /// # Panics:
    /// If the conditions above are not met.
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Vec<B>;

    // Vec uses retain and not filter,
    // but retain_map reads like keep, then replace, which doesn't make sense.

    /// Shorter than `.into_iter().filter_map(f).collect::<Vec<_>>()`,
    /// and faster if the types have identical alignment and the size of `A` is
    /// divisible by the size of `B`: Then the allocation is reused.
    ///
    /// This function doesn't (yet) attempt to optimize cases
    /// where the size of `A` is a multiple of the size of `B`.
    fn filter_map<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B>;

    /// Reuse the memory owned by `self` when filtering and converting
    /// the elements to a different type.
    /// For this to be safe the types must have identical alignment and
    /// the size of `A` must be divisible by the size of `B`
    /// (`size_of::<A>() % size_of::<B>() == 0).
    ///
    /// # Panics:
    /// If the conditions above are not met.
    fn filter_map_in_place<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B>;
}

// VecDeque lacks a from_raw_parts and a way to get/set start and end index.



fn handle_unwind_of<R, F:FnOnce()->R, D:FnMut()>
(might_panic: F,  mut cleanup: D) -> R {
    let guard = guard((), |_| cleanup() );
    let r = might_panic();
    mem::forget(guard);
    r
}



macro_rules! size {($t:ty) => {mem::size_of::<$t>()}}
macro_rules! align {($t:ty) => {mem::align_of::<$t>()}}
macro_rules! sizes {($a:ty, $b:ty, $zero:expr, $alignment:expr,
                     $f:expr => $incompatible:expr, $ok:expr,) => {
    unsafe {
        if size!($a) == 0  ||  size!($b) == 0 {
            $zero
        } else if align!($a) != align!($b) {
            $alignment
        } else if $f(size!($a),size!($b)) {
            $incompatible
        } else {
            $ok
        }
    }
}}
macro_rules! panicy {($a:ty, $b:ty, $rel:expr=>$ok:expr, $notrel:expr) => {
    sizes!{$a,$b,
        panic!("{}", ERR_ZERO_SIZED),
        panic!("{}", ERR_ALIGNMENT),
        $rel => $ok,
        panic!("{}", $notrel),
    }
}}
macro_rules! fallback {($a:ty, $b:ty, $rel:expr=>$ok:expr, $fallback:expr) => {
    sizes!{$a,$b,
        $fallback,
        $fallback,
        $rel => $ok,
        $fallback,
    }
}}



unsafe fn filter_map_vec<A, B, F:FnMut(A)->Option<B>>
(mut vec: Vec<A>,  mut f: F) -> Vec<B> {
    let len = vec.len();
    let cap = vec.capacity() * size!(A)/size!(B);
    let read = vec.as_mut_ptr();
    let write = read as *mut B;
    mem::forget(vec);
    let mut wrote = 0;
    for i in 0..len {
        let a = ptr::read(read.offset(i as isize));
        let result = handle_unwind_of(|| f(a), || {
            for ii in i+1..len {
                ptr::drop_in_place(read.offset(ii as isize));
            }
            mem::drop(Vec::from_raw_parts(write, wrote, cap));
        });
        if let Some(b) = result {
            ptr::write(write.offset(wrote as isize), b);
            wrote += 1;
        }
    }
    Vec::from_raw_parts(write, wrote, cap)
}
unsafe fn map_vec<A, B, F:FnMut(A)->B>
(vec: Vec<A>,  mut f: F) -> Vec<B> {
    filter_map_vec(vec, |a| Some(f(a)) )
}
impl<A> MapVecInPlace<A> for Vec<A> {
    #[inline]
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Vec<B> {
        fallback!{A,B,
            |a,b| a%b==0 => map_vec(self, f),
            self.into_iter().map(f).collect()
        }
    }
    #[inline]
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Vec<B> {
        panicy!{A,B,
            |a,b| a%b==0 => map_vec(self, f),
            ERR_NEED_DIVISIBLE_SIZE
        }
    }
    #[inline]
    fn filter_map<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B> {
        fallback!{A,B,
            |a,b| a%b==0 => filter_map_vec(self, f),
            self.into_iter().filter_map(f).collect()
        }
    }
    #[inline]
    fn filter_map_in_place<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B> {
        panicy!{A,B,
            |a,b| a%b==0 => filter_map_vec(self, f),
            ERR_NEED_DIVISIBLE_SIZE
        }
    }
}



unsafe fn map_slice<A, B, F:FnMut(A)->B>
(boxed: Box<[A]>, f: F) -> Box<[B]> {
    map_vec(boxed.into_vec(), f).into_boxed_slice()
}
impl<A> MapSliceInPlace<A> for Box<[A]> {
    #[inline]
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]> {
        fallback!{A,B,
            |a,b| a==b => map_slice(self, f),
            Vec::into_boxed_slice(self.into_vec().into_iter().map(f).collect())
        }
    }
    #[inline]
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]> {
        panicy!{A,B,
            |a,b| a==b => map_slice(self, f),
            ERR_NEED_EXACT_SIZE
        }
    }
}



unsafe fn map_box<A, B, F:FnOnce(A)->B>
(boxed: Box<A>, f: F) -> Box<B> {
    let aptr = Box::into_raw(boxed);
    let a = ptr::read(aptr);
    let b = handle_unwind_of(|| f(a), || {
        // Currently OK.
        mem::drop(Vec::from_raw_parts(aptr, 0, 1));
    });
    let bptr = aptr as *mut B;
    ptr::write(bptr, b);
    Box::from_raw(bptr)
}
impl<A> MapBoxInPlace<A> for Box<A> {
    #[inline]
    fn map<B,F:FnOnce(A)->B>(self, f: F) -> Box<B> {
        fallback!{A,B,
            |a,b| a==b => map_box(self, f),
            Box::from(f(*self))
        }
    }
    #[inline]
    fn map_in_place<B,M:FnOnce(A)->B>(self, f: M) -> Box<B> {
        panicy!{A,B,
            |a,b| a==b => map_box(self, f),
            ERR_NEED_EXACT_SIZE
        }
    }
}
