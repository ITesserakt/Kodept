use crate::Execution;
use crate::prelude::SourceFiles;
use std::sync::Arc;

pub trait Compiler {
    type Output;

    fn from_sources(sources: Arc<SourceFiles>) -> Execution<Self::Output>;
}

pub trait Interpreter<Program> {
    type Input;
    type State;

    fn initial_state(input: Self::Input) -> Self::State;
    fn join(state: Self::State, program: Program) -> Execution<Self::State>;

    fn run(input: Self::Input, program: Program) -> Execution<Self::State> {
        Self::join(Self::initial_state(input), program)
    }
}
