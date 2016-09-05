extern crate map_in_place;
use map_in_place::{MapVecInPlace, MapSliceInPlace, MapBoxInPlace};


fn nv<T:Copy>(fill:T, ) -> (Vec<T>,usize) {
    let v = vec![fill, fill, fill, fill];
    let p = v.as_ptr() as usize;
    (v,p)
}
fn nfv<T,F:FnMut(u32)->T>(f:F) -> (Vec<T>,usize) {
    let v = (0..4).map(f).collect::<Vec<_>>();
    let p = v.as_ptr() as usize;
    (v, p)
}

#[test]
fn vec_eq_success() {
    let (v,p) = nv('\0');
    let cap = v.capacity();
    let v = v.map_in_place(|e| e as u32 );
    assert_eq!(&v, &[0,0,0,0]);
    assert_eq!(v.capacity(), cap);
    assert_eq!(v.as_ptr() as usize, p);
}
#[test]
fn vec_third_success() {
    let (v,p) = nv([1u8;3]);
    let cap = v.capacity();
    let v = v.map_in_place(|e| e[0] );
    assert_eq!(&v, &[1,1,1,1]);
    assert_eq!(v.capacity(), cap*3);
    assert_eq!(v.as_ptr() as usize, p);
}
#[test]
fn vec_filter_third_success() {
    let (v,p) = nfv(|e| [e as u8; 3] );
    let cap = v.capacity();
    let v = v.filter_map_in_place(|e| if e[0]>=2 {Some(e[0])} else {None} );
    assert_eq!(&v, &[2,3]);
    assert_eq!(v.capacity(), cap*3);
    assert_eq!(v.as_ptr() as usize, p);
}
#[test]
fn vec_filter_fallback() {
    let (v,p) = nfv(|e|e);
    let v = v.filter_map(|e| if e>=2 {Some(e as u64)} else {None});
    assert_eq!(&v, &[2,3]);
    assert!(v.as_ptr() as usize != p);
}
#[test]
fn vec_fallback() {
    let (v,p) = nv(1u8);
    let v = v.map(|e| e as u16 );
    assert_eq!(&v, &[1,1,1,1]);
    assert!(v.as_ptr() as usize != p);
}
#[test] #[should_panic(expected="`A` and `B` have different alignment")]
fn vec_alignment() {let _ = nv((1u8,2u8)).0.map_in_place(|(a,b)| ((a as u16)<<8) | b as u16 );}
#[test] #[should_panic(expected="The size of `A` is not equal to or a multiple of the size of `B`")]
fn vec_bigger() {let _ = nv(1u8).0.map_in_place(|e| [e,e] );}
#[test] #[should_panic(expected="The size of `A` is not equal to or a multiple of the size of `B`")]
fn vec_smaller() {let _ = nv([1u8;3]).0.map_in_place(|a| [a[0],a[1]] );}


fn ns<T:Copy>(fill:T) -> (Box<[T]>,usize) {
    let (v,p) = nv(fill);
    (v.into_boxed_slice(), p)
}

#[test]
fn slice_success() {
    let (bs,p) = ns('\0');
    let bs = bs.map_in_place(|e| e as u32 );
    assert_eq!(&bs[..], &[0,0,0,0]);
    assert_eq!(bs.as_ptr() as usize, p);
}
#[test]
fn slice_fallback() {
    let (bs,p) = ns(1u8);
    let bs = bs.map(|e| e as u16 );
    assert_eq!(&bs[..], &[1,1,1,1]);
    assert!(bs.as_ptr() as usize != p);
}
#[test] #[should_panic(expected="`A` and `B` have different alignment")]
fn slice_alignment() {let _ = ns((1u8,2u8)).0.map_in_place(|(a,b)| ((a as u16)<<8) | b as u16 );}
#[test] #[should_panic(expected="`A` and `B` have different size")]
fn slice_bigger() {let _ = ns(1u8).0.map_in_place(|e| [e,e] );}
#[test] #[should_panic(expected="`A` and `B` have different size")]
fn slice_half() {let _ = ns([1u8,2]).0.map_in_place(|a| a[0] );}


#[test]
fn box1_success() {
    let b = Box::new('\0');
    let p = b.as_ref() as *const _ as usize;
    let b = b.map_in_place(|c| c as u32 );
    assert_eq!(*b, 0);
    assert_eq!(b.as_ref() as *const _ as usize, p);
}
#[test]
fn box1_fallback() {
    let b = Box::new(0_u8);
    let p = b.as_ref() as *const _ as usize;
    let b = b.map(|n| n as char );
    assert_eq!(*b, '\0');
    assert!(b.as_ref() as *const _ as usize != p);
}
#[test] #[should_panic(expected="`A` and `B` have different alignment")]
fn box1_alignment() {let _: Box<u32> = Box::new([1_u8; 4]).map_in_place(|_| 20 );}
#[test] #[should_panic(expected="`A` and `B` have different size")]
fn box1_bigger() {let _ = Box::new([1_u8; 4]).map_in_place(|_| [2_u8; 5] );}
#[test] #[should_panic(expected="`A` and `B` have different size")]
fn box1_half() {let _ = Box::new([1_u8; 4]).map_in_place(|_| [2_u8; 2] );}
