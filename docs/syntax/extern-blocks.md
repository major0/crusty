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
    pub fn malloc(size: usize) -> *mut ();
    pub fn free(ptr: *mut ());
}
```

> **Variadic functions** (`...`, as in C's `printf`) are not yet implemented.
> Declare a variadic foreign function with a `__rust__{ }` block for now.

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

## Planned

The following are designed but **not yet implemented**:

- **Variadic parameters** (`...`) in an extern function declaration.
- **Extern blocks inside a function body** (only module-level extern blocks parse today).
- **`__rust__{ }` escape hatch inside an extern block**, for signatures Crusty
  cannot express (function pointers with ABI annotations, `static mut` globals):

```c
extern "C" {
    void simple_function();

    __rust__{
        fn complex_function(callback: extern "C" fn(i32) -> i32) -> i32;
        static mut GLOBAL_STATE: i32;
    }
}
```

  With the planned nesting rules: `__rust__{ }` within extern allowed; extern
  within extern disallowed; nested `__rust__{ }` disallowed.

## Formal Grammar

```ebnf
extern_block = "extern" [string_literal] "{" extern_fn_decl* "}" ;
extern_fn_decl = type IDENT "(" param_list ")" ";" ;
```
