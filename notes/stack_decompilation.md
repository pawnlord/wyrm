# stack-based language decompilation
For wyrm, I've been doing a lot of thinking about how a stack-based language should be much easier to decompile compared to a register-based language. I'm going to share some of those thoughts here.

## statement finding
One issue with standard decompilation is finding when something is a statement: e.g., how can we know if
```asm
mov rbx, 3
mov rax, 2
mov rcx, [MEM_LOCATION_n]
mul rcx, rbx
add rcx, rax
mov [MEM_LOCATION_out], rcx
```
will be decompiled to 
```c
int out = 3 * n + 2;
```
or 
```c
int temp = 3 * n + 2;
...
out = temp;
```
without knowing about the rest of the program? With a stack-based machine code, we _know_ the second option can't happen. When it is stored in a local or global variable, the "register" (stack location) is discarded. Another consequence of this fact is that we can know if something is a statement or not: if, at the end of the section, the stack has a net change of 0, we know it's a statement. if it has a higher net change, then it is probably not a statement: the only way for it to be a statement is if the rest of the function never uses it's result, in which case at least one operation did something unnecessary in the section.  
