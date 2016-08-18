# map_in_place

Reuse alloations when mapping the elements of a `Vec`, `Box<[T]>` or `Box<T>`
if possible.

To map in place the types must have identical alignment and:
* for boxes and boxed slices the sizes must be equal,
* for vectors the size of *in* must be a multiple of the *out* type.
  (so *out* cannot be bigger than *in*)

The `..._in_place()` methods will panic if not possible,
while the others will fall back to iterating and collecting.

## Example

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

## Why all those restrictions?  
The rust allocation interface is a bit more complex than the standard C one of
`malloc(size_t)` and `free(void*)`:

First, malloc and free takes the alignment of the types you want to store, and allocating with one alignment and freeing with another is undefined behaviour.

Second, rust requires the owner to know the size of the memory to free, which means one of the types' size must be a multiple of the other, since the capacity is an integer.  

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
