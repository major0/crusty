# Extern Blocks (FFI)

## Introduction

Crusty supports extern blocks for Foreign Function Interface (FFI) declarations. Extern blocks allow calling functions from C libraries, system calls, and other foreign code. The syntax mirrors Rust's extern blocks but uses Crusty-style function declarations inside them.

## Rationale

FFI is essential for systems programming. Crusty preserves Rust's extern block syntax for ABI specification while allowing Crusty-style function declarations inside the blocks, maintaining consistency with the rest of the language.

## Examples

### C ABI (Most Common)
```c
extern "C" {
    int puts(char* s);
    void* malloc(size_t size);
    void free(void* ptr);
}
```

Translates to:
```rust
extern "C" {
    pub fn puts(s: *const char) -> i32;
    pub fn malloc(size: usize) -> *const ();
    pub fn free(ptr: *const ());
}
```

> **Variadic functions** (`...`, as in C's `printf`) are supported. See
> [Variadic Declarations](#variadic-declarations) below.

### Variadic Declarations
A trailing `...` marks a C-style variadic foreign function. It follows the named
parameters (or stands alone), exactly as in C:
```c
extern "C" {
    int printf(char* format, ...);
    void log_event(...);
}
```

Translates to:
```rust
extern "C" {
    pub fn printf(format: *const char, ...) -> i32;
    pub fn log_event(...);
}
```

The `...` is not a named parameter — it is a signature-level flag. When named
parameters precede it, a separating comma is emitted (`format: *const char, ...`);
a bare `(...)` emits no leading comma. Variadics are only valid on body-less
`extern` declarations, matching Rust, which forbids them on defined functions.

### Raw Rust Escape Hatch
For FFI signatures Crusty cannot express — function pointers with ABI
annotations, `static mut` globals — a `__rust__{ … }` block inside an extern
block passes its contents through to the generated Rust **verbatim**:
```c
extern "C" {
    void simple_function();

    __rust__{
        fn complex_function(callback: extern "C" fn(i32) -> i32) -> i32;
        static mut GLOBAL_STATE: i32;
    }
}
```

Translates to (the `__rust__` contents emitted untouched, indented in place):
```rust
extern "C" {
    pub fn simple_function();

    fn complex_function(callback: extern "C" fn(i32) -> i32) -> i32;
    static mut GLOBAL_STATE: i32;
}
```

The block body is brace-balanced and is **not parsed** by Crusty — rustc is the
only arbiter of its validity. Nesting rules follow from the grammar: a
`__rust__{ }` block is valid only inside an extern block; an extern block
cannot nest inside another; and because the body is opaque, a `__rust__` block
cannot contain a parsed extern block or another `__rust__` block.

### Default ABI (Rust)
```c
extern {
    void some_rust_function();
}
```

### System ABI
```c
extern "system" {
    void SystemCall(int param);
}
```

## Supported ABI Strings

All Rust ABI specifications are supported: `"C"`, `"cdecl"`, `"stdcall"`, `"fastcall"`, `"system"`, `"Rust"`, and others. The ABI string is preserved exactly as specified in the generated Rust code.

## Key Points

- Function declarations inside extern blocks use Crusty syntax (return type before name)
- The ABI string is optional — omitting it defaults to the Rust ABI (bare `extern`)
- Extern functions are typically unsafe to call in the generated Rust code
- Extern blocks are supported at module (top) level
- A `__rust__{ … }` block passes raw Rust through verbatim for signatures Crusty cannot express

## Planned

The following are designed but **not yet implemented**:

- **Extern blocks inside a function body** (only module-level extern blocks parse today).

## Formal Grammar

```ebnf
extern_block   = "extern" [string_literal] "{" extern_item* "}" ;
extern_item    = rust_escape | extern_fn_decl ;
rust_escape    = "__rust__" "{" <brace-balanced raw Rust> "}" ;
extern_fn_decl = (type | "void") IDENT "(" [param_list] ["," "..."] ")" ";" ;
```
