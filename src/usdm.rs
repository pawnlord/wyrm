/*
* Universal stack decompilation machine
* An abstract representation of actions taken by the program
* Responsibilities: data flow analysis, program flow analysis
* This is stack-based bytecode specific as we want it to handle
* "registers"
*/

use crate::instr_table::get_instr;

pub enum SpecialStackOp<T: UsdmSegment> {
    None,
    CreateVar(T::Type),
}

pub struct StackOperation<T: UsdmSegment> {
    pub in_types: Vec<T::Type>,
    pub out_types: Vec<T::Type>,
    pub special_op: SpecialStackOp<T>,
}

impl<T: UsdmSegment> StackOperation<T> {
    pub fn new() -> Self {
        Self {
            in_types: vec![],
            out_types: vec![],
            special_op: SpecialStackOp::<T>::None,
        }
    }
}

pub trait UsdmSegment: Clone {
    type Type;

    fn get_stack_operation(&self) -> StackOperation<Self>;
}

pub trait UsdmFrontend: Clone {
    type Type: Clone;
    type Segment: UsdmSegment<Type = Self::Type>;
    type SegmentIterator<'a>: Iterator<Item = &'a Self::Segment>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Self::SegmentIterator<'a>;
}

#[derive(Clone)]
pub enum UsdmExpression<T: UsdmFrontend> {
    UsdmVariable {
        _type: T::Type,
    },
    UsdmExpr {
        operation: T::Segment,
        out_num: usize,
        arguments: Vec<UsdmExpression<T>>,
    },
}

impl<T: UsdmFrontend> UsdmExpression<T> {
    pub fn empty(_type: T::Type) -> Self {
        Self::UsdmVariable { _type }
    }
}

// This state is separate from any underlying machine:
// It is the current state of the stack, as found by analysis.
pub struct UsdmState<T: UsdmFrontend> {
    pub stack: Vec<UsdmExpression<T>>,
    pub stack_capacity: usize,
}

pub struct UsdmOptions {}

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
    final_state: UsdmState<T>,
}

impl<T: UsdmFrontend> Usdm<T> {
    pub fn new(frontend: T) -> Self {
        Self {
            expr_string: frontend,
            timeline: Vec::new(),
            options: UsdmOptions {},
            final_state: UsdmState {
                stack: Vec::new(),
                stack_capacity: 0,
            },
        }
    }

    // The main analysis function
    pub fn analyze_data(&mut self) {
        for seg in self.expr_string.clone().iter() {
            let stack_op = seg.get_stack_operation();
            let stack_growth = stack_op.out_types.len() - stack_op.in_types.len();
            let mut vars: Vec<UsdmExpression<T>> = vec![];

            for var_in in stack_op.in_types {
                vars.push(self.final_state.stack.pop().expect(
                    "Too many inputs for current stack size: Check if this is actually WASM code",
                ));
            }
            match stack_op.special_op {
                SpecialStackOp::None => {
                    for (i, var_out) in stack_op.out_types.iter().enumerate() {
                        self.final_state.stack.push(UsdmExpression::UsdmExpr {
                            operation: seg.clone(),
                            out_num: i,
                            arguments: vars.clone(),
                        });
                    }
                }
                SpecialStackOp::CreateVar(_type) => {
                    self.final_state.stack.push(UsdmExpression::empty(_type));
                }
            }
        }
    }
}
