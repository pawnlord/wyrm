# wyrm
WASM Decompiler

## Goals
This decompiler was inspired by a CTF challenge I did recently where I realized that the current tooling around WASM reversing is sorely lacking, despite WASM being a fairly digestible language in terms of reverse engineering. The current state of the art, the [WebAssembly Binary Toolkit](https://github.com/WebAssembly/wabt), lacks much of the ability, including failing to notice simple reductions sometimes, such as redundant assignment. E.g., for the CTF I was doing, the decompiler couldn't simplify:
```js
var p:int = 208;
var q:int = stack + p;
var r:int = q;
```
To the much more readable, and likely more semantically correct:
```js
var r:int = stack + 208;
```
Things like this seem to happen in any sufficiently large code base. It also produces a bespoke language, which is not used anywhere else. The C decompiler option fails on even the simplest of examples, and as such there is no easy starting point for decompiling besides learning the semantics of the language they're using. This includes things like a function `eqz(a)`, instead of how most languages would right that: `a == 0`.  
Because of these reasons, the goals of this project are as follows:
- Provide a decompiler to a in-use language that properly reduces statements to maximize readability (with options for how much this should impact actual program correctness)
- Provide a decompiler with loop inference in mind.
- Provide an intermediate representation of the decompilation for easy portability to other languages. 
