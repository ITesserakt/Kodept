use std::sync::Arc;
use crate::Execution;
use crate::prelude::SourceFiles;

pub trait Compiler<SourceImpl> {
    type Output;
    
    fn from_sources(sources: Arc<SourceFiles<SourceImpl>>) -> Execution<Self::Output>;
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
