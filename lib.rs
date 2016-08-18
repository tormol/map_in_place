/*!
Reuse alloations when mapping the elements of a `Vec`, `Box<[T]>` or `Box<T>`
if possible.

To map in place the types must have identical alignment and:
* for boxes and boxed slices the sizes must be equal,
* for vectors the size of *in* must be a multiple of the *out* type.
  (so *out* cannot be bigger than *in*)

The `..._in_place()` methods will panic if not possible,
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

fn handle_panic_of<R, F:FnOnce()->R, D:FnMut()>
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
        panic!("The optimization doesn't make sense fore zero-sized types"),
        panic!("`A` and `B` have different alignment"),
        $rel => $ok,
        panic!($notrel),
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



// VecDeque lacks a from_raw_parts and a way to get/set start and end index.



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
        let result = handle_panic_of(|| f(a), || {
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
pub trait MapVecInPlace<A> {
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Vec<B>;
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Vec<B>;
    // Vec uses retain and not filter,
    // but retain_map reads like keep, then replace, which doesn't make sense.
    fn filter_map<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B>;
    fn filter_map_in_place<B,F:FnMut(A)->Option<B>>(self, f: F) -> Vec<B>;
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
            "The size of `A` is not a multiple of `B`"
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
            "The size of `A` is not a multiple of `B`"
        }
    }
}



unsafe fn map_slice<A, B, F:FnMut(A)->B>
(boxed: Box<[A]>, f: F) -> Box<[B]> {
    map_vec(boxed.into_vec(), f).into_boxed_slice()
}
pub trait MapSliceInPlace<A> {
    fn map<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]>;
    fn map_in_place<B,F:FnMut(A)->B>(self, f: F) -> Box<[B]>;
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
            "`A` and `B` have different size"
        }
    }
}



unsafe fn map_box<A, B, F:FnOnce(A)->B>
(boxed: Box<A>, f: F) -> Box<B> {
    let aptr = Box::into_raw(boxed);
    let a = ptr::read(aptr);
    let b = handle_panic_of(|| f(a), || {
        // Currently OK.
        mem::drop(Vec::from_raw_parts(aptr, 0, 1));
    });
    let bptr = aptr as *mut B;
    ptr::write(bptr, b);
    Box::from_raw(bptr)
}
pub trait MapBoxInPlace<A> {
    fn map<B,F:FnOnce(A)->B>(self, f: F) -> Box<B>;
    fn map_in_place<B,F:FnOnce(A)->B>(self, f: F) -> Box<B>;
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
            "`A` and `B` have different size"
        }
    }
}
