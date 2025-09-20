use vexide::{
    prelude::*,
    program::{CodeSignature, ProgramOptions, ProgramOwner, ProgramType},
};

// The custom code signature can be used to configure program behavior.
static CODE_SIG: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::System,
    ProgramOptions::INVERT_DEFAULT_GRAPHICS,
);

#[vexide::main(code_sig = CODE_SIG)]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");
}
