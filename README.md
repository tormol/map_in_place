# map_in_place

Reuse allocations when converting the elements of a vector, boxed slice or box
to a compatible type.

To map in place the types must have identical alignment and size.
For vectors the size of *in* can also be a multiple of the size of *out*
because it can change the capacity. (but *out* can never be bigger than *in*).

The `*_in_place()` methods will panic if the types are not compatible,
while the others will fall back to iterating and collecting.

## Example

```rust
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
The Rust allocation interface is a bit more complex than the standard C one of
`malloc(size_t)` and `free(void*)`:

First, `alloc()` and `dealloc()` takes the alignment of the types you want to store,
and allocating with one alignment and freeing with another is undefined behaviour.

Second, Rust requires the owner to know the size of the memory to free,
which means one of the types' size must be a multiple of the other,
since the capacity is an integer.

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
