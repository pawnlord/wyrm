/*
* Universal stack decompilation machine
* An abstract representation of actions taken by the program
* Responsibilities: data flow analysis, program flow analysis
* This is stack-based bytecode specific as we want it to handle
* "registers"
*/





pub trait UsdmFrontend {
   type Type;
   type Segment;
}



pub struct UsdmVariable<T: UsdmFrontend> {
   pub _type: T::Type,
}

// This state is separate from any underlying machine:
// It is the current state of the stack, as found by analysis.
pub struct UsdmState<T: UsdmFrontend> {
   pub stack: Vec<UsdmVariable<T>>
}

// T represents the underlying information of the USDM, e.g. the instructions
//   of the machine we are decompiling from
// The idea is two-fold: we never want to lose that information in the decompilation
//   process (we always want to get it back, for debugging purposes) and, from the 
//   perspective of both the USDM and the backend, we only want to know things
//   from the frontend that are needed at the time: if we want to know if a set of
//   instructions represents an `(add a b)` command, we don't want to know the types
//   of a and b, that's an implementation detail: we just want to know if there's
//   an add instruction interacting with 2 variables
pub struct Usdm<T: UsdmFrontend> {
   expr_string: Vec<T::Segment>,
   timeline: Vec<UsdmState<T>>   
} 