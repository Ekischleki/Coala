pub enum Feature {
    Assert, //Using assertations to throw runtime errors
    Format, //Runtime string formatting
    Cycle(Vec<CycleOptions>),
    Recursion {
        cycled: bool, //Whether to only allow recursion when at least one cycle has passed
    },
    Loops {
        cycled: bool, //Whether to only allow loops when a set amount of cycles have passed
    },
    Integer
}

pub enum IntegerOptions {

}

pub enum CycleOptions {
    Memory, //Using memory which persists between cycles
    VariableCycles, //Allows functions to wait for a variable amount of cycles
}

pub trait Architecture {
    fn name(&self) -> &str;
    fn supported(&self) -> &[Feature];
}

pub enum FunctionCall {
    Assert(),
    Bool(),
    Integer(),
}

pub enum AssertFunction {
    IsTrue,
    IsFalse,
}