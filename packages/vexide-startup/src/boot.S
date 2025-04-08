@ vexide's first-stage boot routine.
@
@ This is the true entrypoint of vexide, containing the first instructions
@ of user code executed before anything else. This is written in assembly to
@ ensure that it stays the same across compilations (a requirement of the patcher),
@
@ This routine loads the stack pointer to the stack region specified in our
@ linkerscript, makes a copy of program memory for the patcher if needed, then
@ branches to the Rust entrypoint (_start) created by the #[vexide::main] macro.

.section .boot, "ax"
.global _boot

_boot:
    @ Load the stack pointer to point to our stack section.
    @
    @ This technically isn't required, as VEXos already sets up a stack for CPU1,
    @ but that stack is relatively small and we have more than enough memory
    @ available to us for this.
    @
    @ SAFETY: Doing this should be safe, since VEXos doesn't seem to use its existing
    @ stack after calling user code. This operation is safe assuming that the variables
    @ on the previous stack are never read or written to during execution of the program.
    ldr sp, =__stack_top

    @ Before any Rust code runs, we need to memcpy the currently running program in
    @ memory to the `.patcher_base` section if a patch file needs to be applied.
    @
    @ We do this since we need an unmodified copy of the base binary to run the
    @ patcher correctly. Since Rust code will modify the binary by writing to `.data`
    @ and `.bss`, we can't reference program memory in the patcher so we must make a
    @ carbon copy of it before any Rust code gets the chance to modify these sections.

    @ Check if a patch file is loaded into memory by reading the first four bytes
    @ at the expected location (0x07A00000) and checking if they equal the magic
    @ header value of 0xB1DF.
    mov r0, #0x07A00000
    ldr r0, [r0]
    ldr r1, =0xB1DF
    cmp r0, r1 @ r0 == 0xB1DF?

    @ Prepare to memcpy binary to 0x07C00000

    mov r0, #0x07C00000 @ memcpy dest -> r0
    mov r1, #0x03800000 @ memcpy src -> r1
    ldr r2, =0x07A0000C @ the length of the binary is stored at 0x07A0000C
    ldr r2, [r2]        @ memcpy size -> r2

    @ Do the memcpy if patch magic is present (we checked this in our `cmp` instruction).
    bleq __overwriter_aeabi_memcpy

    @ Jump to the Rust entrypoint.
    b _start
