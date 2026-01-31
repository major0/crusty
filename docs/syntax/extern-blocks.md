# Extern Blocks (FFI)

## Introduction

Crusty supports extern blocks for Foreign Function Interface (FFI) declarations. Extern blocks allow calling functions from C libraries, system calls, and other foreign code. The syntax mirrors Rust's extern blocks but uses Crusty-style function declarations inside them.

## Rationale

FFI is essential for systems programming. Crusty preserves Rust's extern block syntax for ABI specification while allowing Crusty-style function declarations inside the blocks, maintaining consistency with the rest of the language.

## Examples

### C ABI (Most Common)
```c
extern "C" {
    void printf(char* format, ...);
    int puts(char* s);
    void* malloc(size_t size);
    void free(void* ptr);
}
```

Translates to:
```rust
extern "C" {
    pub fn printf(format: *const i8, ...);
    pub fn puts(s: *const i8) -> i32;
    pub fn malloc(size: usize) -> *mut ();
    pub fn free(ptr: *mut ());
}
```

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

### Using Extern Functions
```c
void main() {
    extern "C" {
        int getpid();
    }

    let pid = getpid();
    __println__("Process ID: {}", pid);
}
```

## Supported ABI Strings

All Rust ABI specifications are supported: `"C"`, `"cdecl"`, `"stdcall"`, `"fastcall"`, `"system"`, `"Rust"`, and others. The ABI string is preserved exactly as specified in the generated Rust code.

## Key Points

- Function declarations inside extern blocks use Crusty syntax (return type before name)
- The ABI string is optional — omitting it defaults to Rust ABI
- Extern functions are typically unsafe to call in the generated Rust code
- Extern blocks are supported at module level and inside functions

## Formal Grammar

```ebnf
extern_block = "extern" [string_literal] "{" extern_fn_decl* "}" ;
extern_fn_decl = type IDENT "(" param_list ")" ";" ;
```
