# Arcane Knowledge
This is a store of knowledge that is not well documented for WebAssembly (or that I personally found confusing), for which I have had to guess at, either through reversing it from a dump or from reading through multiple second hand sources.  

## unsized values
Some values, such as the length of a section, are not of a specific size. They can be any number of bytes, I guess for the purpose of limiting the size of the final binary. For these values, they highest bit in a byte tells you if you need to read another value. For example, if you have `0x8a01`, then, looking at the bytes:  
```c
1|0|0|0 1|0|1|0        ---> 0x8a
^ This bit being set means read the next byte
  to find the actual value, little endian.
0|0|0|0 0|0|0|1        ---> 0x01
----------------------------------
final value:
0|0|0|0 0|0|0|1 0|0|0|0 1|0|1|0        ---> 0x10A
                ^ information bit unset
``` 
Note that this encoding/decoding strategy is only for unsigned integers. Signed integers have the second to last bit set to 1



Some places where these "unsized values" are used:
- The size of sections
- The number of items present in a section
- (i64|i32).const. This does not apply to float constants, those are a fixed 32 or 64 bits.

## global data structure
```
; section "Global" (6)
0000136: 06                                        ; section code
0000137: 00                                        ; section size (guess)
0000138: 04                                        ; num globals
```
This is the header of the global section, from a wat2wasm dump. It is fairly self explanatory.
```
0000139: 7f                                        ; i32
000013a: 01                                        ; global mutability
000013b: 41                                        ; i32.const
000013c: 8080 04                                   ; i32 literal
000013f: 0b                                        ; end
```
This is one global definition. The first byte is the type. The encoding that I've found is:  
```rust
0x7c -> f64,
0x7d -> f32,
0x7e -> i64,
0x7f -> i32
```
This will be updated as I look at dumps from more examples.  
The second byte tells us whether or not this global is mutable. The rest of this struct is an expression, up to the `end` instruction, and the top of the stack after the expression is evaluated is the value of the global.  
The i32.const does not have anything to do with the mutability. That is the main reason this section was written for this document. 

## knowledge needed
The element section has something called "elemkind", which does not redirect to any docs.  
It is assumed that a funcidx is an offset into the function table.
