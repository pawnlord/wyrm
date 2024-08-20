/*
* Universal stack decompilation machine
* An abstract representation of actions taken by the program
* Responsibilities: data flow analysis, program flow analysis
* This is stack-based bytecode specific as we want it to handle
* "registers"
*/

use crate::instr_table::get_instr;

pub struct StackOperation<T> {
    pub in_types: Vec<T>,
    pub out_types: Vec<T>,
}

pub trait UsdmSegment: Clone {
    type Type;

    fn get_stack_operation(&self) -> StackOperation<Self::Type>;
}

pub trait UsdmFrontend : Clone {
    type Type;
    type Segment: UsdmSegment<Type = Self::Type>;
    type SegmentIterator<'a> : Iterator<Item = &'a Self::Segment> where Self : 'a;

    fn iter<'a>(&'a self) -> Self::SegmentIterator<'a>;
}



pub struct UsdmVariable<T: UsdmFrontend> {
    pub _type: T::Type,
}

// This state is separate from any underlying machine:
// It is the current state of the stack, as found by analysis.
pub struct UsdmState<T: UsdmFrontend> {
    pub stack: Vec<UsdmVariable<T>>,
    pub stack_capacity: usize
}

pub struct UsdmOptions {

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
    expr_string: T,
    timeline: Vec<UsdmState<T>>,
    options: UsdmOptions,
    final_state: UsdmState<T>
}

impl<T: UsdmFrontend> Usdm<T> {
    pub fn new(frontend: T) -> Self {
        Self {
            expr_string: frontend,
            timeline: Vec::new(),
            options: UsdmOptions {},
            final_state: UsdmState { stack: Vec::new(), stack_capacity: 0 }
        }
    }

    // The main analysis function
    pub fn analyze(&mut self) {
        for seg in self.expr_string.clone().iter() {
            let stack_op = seg.get_stack_operation();
            let stack_growth = stack_op.out_types.len() - stack_op.in_types.len();
            for var_in in stack_op.in_types {
            }

            for var_out in stack_op.out_types {

            }         
        }
    }
}