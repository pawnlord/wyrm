# wyrm
WASM Reverse engineering tool

## purpose
I wanted to learn about both WASM and decompilers, and WASM seemed like a good place to start due to it being a stack-based language. The goal of this project is to provide a base for decompiling and reversing stack-based languages specifically (and may be expanded to other bytecodes, such as compiled python). It is currently for educational purposes, so may not break new ground so to say.

## implemented features
- WASM file parser
- WASM instruction set and expression representation
- WASM WAT emitter, close to parity with the WAT emitters built for WASM.