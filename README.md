# wyrm
WASM Decompiler

## Goals
I wanted to learn how decompilation works, and WASM, being a stack-based VM, seemed like an easy but practical place to start. Also, the only way to decompile WASM that I've been able to find are extensions for the big decompilers, so I thought it would be interesting to do it from scratch. Most of these decompilers, including Ghidra, have a hard time with how WASM represents variables, sometimes failing to simplify code like:  
```js
var p:int = 208;
var q:int = stack + p;
var r:int = q;
```


## Currently Implemented
- Reading from a WASM file
- Writing some amount of WAT disassembly