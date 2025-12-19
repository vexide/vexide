// This file is written in C instead of Rust so that it can access
// `FDEV_SETUP_STREAM`, a macro whose implementation details are intended
// to be private. It also acts as a built-in check that C compilation is
// set up correctly.

#include <errno.h>
#include <stdio.h>

// errno is an extern `thread_local` which is unstable in Rust,
// so we have a specialized way of accessing it.
void vexide_set_enomem() {
    errno = ENOMEM;
}

// Everything else, including these functions, are defined from Rust.
extern int vexide_stdio_putc(char ch, FILE* file);
extern int vexide_stdio_getc(FILE* file);
extern int vexide_stdio_flush(FILE* file);

static FILE __stdio = FDEV_SETUP_STREAM(
    vexide_stdio_putc,
    vexide_stdio_getc,
    vexide_stdio_flush,
    _FDEV_SETUP_RW
);

// Redirect stdio as per <https://github.com/picolibc/picolibc/blob/main/doc/os.md>
FILE *const stdin = &__stdio;
extern FILE *const stdout [[gnu::alias("stdin")]];
extern FILE *const stderr [[gnu::alias("stdin")]];
