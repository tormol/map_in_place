extern crate map_in_place;
use map_in_place::{MapVecInPlace, MapSliceInPlace, MapBoxInPlace};

fn nv<T:Copy>(fill:T)->Vec<T> {vec![fill, fill, fill, fill]}
fn nfv<T,F:FnMut(u32)->T>(f:F)->Vec<T> {(0..4).map(f).collect()}
#[test]
fn vec_eq_success() {
    let v = nv('\0');
    let cap = v.capacity();
    let v = v.map_in_place(|e| e as u32 );
    assert_eq!(&v, &[0,0,0,0]);
    assert_eq!(v.capacity(), cap);
}
#[test]
fn vec_third_success() {
    let v = nv([1u8;3]);
    let cap = v.capacity();
    let v = v.map_in_place(|e| e[0] );
    assert_eq!(&v, &[1,1,1,1]);
    assert_eq!(v.capacity(), cap*3);
}
#[test]
fn vec_filter_third_success() {
    let v = nfv(|e| [e as u8; 3] );
    let cap = v.capacity();
    let v = v.filter_map_in_place(|e| if e[0]>=2 {Some(e[0])} else {None} );
    assert_eq!(&v, &[2,3]);
    assert_eq!(v.capacity(), cap*3);
}
#[test]
fn vec_filter_fallback() {
    let v = nfv(|e|e).filter_map(|e| if e>=2 {Some(e as u64)} else {None});
    assert_eq!(&v, &[2,3]);
}
#[test]
fn vec_fallback() {assert_eq!(&nv(1u8).map(|e| e as u16 ), &[1,1,1,1]);}
#[test]#[should_panic(expected="`A` and `B` have different alignment")]
fn vec_alignment() {let _ = nv((1u8,2u8)).map_in_place(|(a,b)| ((a as u16)<<8) | b as u16 );}
#[test]#[should_panic(expected="The size of `A` is not a multiple of `B`")]
fn vec_bigger() {let _ = nv(1u8).map_in_place(|e| [e,e] );}
#[test]#[should_panic(expected="The size of `A` is not a multiple of `B`")]
fn vec_smaller() {let _ = nv([1u8;3]).map_in_place(|a| [a[0],a[1]] );}

fn ns<T:Copy>(fill:T)->Box<[T]> {nv(fill).into_boxed_slice()}
#[test]
fn slice_success() {assert_eq!(&ns('\0').map_in_place(|e| e as u32 )[..], &[0,0,0,0]);}
#[test]
fn slice_fallback() {assert_eq!(&ns(1u8).map(|e| e as u16 )[..], &[1,1,1,1]);}
#[test]#[should_panic(expected="`A` and `B` have different alignment")]
fn slice_alignment() {let _ = ns((1u8,2u8)).map_in_place(|(a,b)| ((a as u16)<<8) | b as u16 );}
#[test]#[should_panic(expected="`A` and `B` have different size")]
fn slice_bigger() {let _ = ns(1u8).map_in_place(|e| [e,e] );}
#[test]#[should_panic(expected="`A` and `B` have different size")]
fn slice_half() {let _ = ns([1u8,2]).map_in_place(|a| a[0] );}

#[test]
fn box1_success() {assert_eq!(*Box::new('\0').map_in_place(|c| c as u32 ), 0);}
#[test]
fn box1_fallback() {assert_eq!(*Box::new(0_u8).map(|n| n as char ), '\0');}
#[test]#[should_panic(expected="`A` and `B` have different alignment")]
fn box1_alignment() {let _: Box<u32> = Box::new([1_u8; 4]).map_in_place(|_| 20 );}
#[test]#[should_panic(expected="`A` and `B` have different size")]
fn box1_bigger() {let _ = Box::new([1_u8; 4]).map_in_place(|_| [2_u8; 5] );}
#[test]#[should_panic(expected="`A` and `B` have different size")]
fn box1_half() {let _ = Box::new([1_u8; 4]).map_in_place(|_| [2_u8; 2] );}
