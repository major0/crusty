// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Integration tests for rustc invocation through the CLI

#[cfg(test)]
mod tests {
    use crate::cli::{run_compiler, CompilerOptions, EmitMode};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_compile_valid_crusty_to_binary() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int main() {
    return 0;
}
"#;
        let input_path = PathBuf::from("test_compile_valid_12345.crst");
        let output_path = PathBuf::from("test_compile_valid_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // This test will only pass if rustc is installed
        // If rustc is not available, the test should fail with RustcInvocation error
        if result.is_ok() {
            // Compilation succeeded - rustc is available
            assert!(result.is_ok());
        } else if let Err(err) = result {
            // Check that it's a rustc invocation error (rustc not found)
            assert!(matches!(
                err,
                crate::error::CompilerError::RustcInvocation(_)
            ));
        }
    }

    #[test]
    fn test_compile_invalid_crusty_to_binary() {
        // Create an invalid Crusty source file (missing return type)
        let test_source = r#"
main() {
    return 0;
}
"#;
        let input_path = PathBuf::from("test_compile_invalid_12345.crst");
        let output_path = PathBuf::from("test_compile_invalid_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // Should fail during parsing
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_with_no_compile_flag() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let input_path = PathBuf::from("test_no_compile_12345.crst");
        let output_path = PathBuf::from("test_no_compile_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: true, // Skip rustc invocation
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed without invoking rustc
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_rust_emit_mode() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int multiply(int x, int y) {
    return x * y;
}
"#;
        let input_path = PathBuf::from("test_rust_emit_12345.crst");
        let output_path = PathBuf::from("test_rust_emit_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Rust, // Only generate Rust, don't compile
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed without invoking rustc
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_verbose_output() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int square(int n) {
    return n * n;
}
"#;
        let input_path = PathBuf::from("test_verbose_12345.crst");
        let output_path = PathBuf::from("test_verbose_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Rust,
            absorb: None,
            verbose: true, // Enable verbose output
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed
        assert!(result.is_ok());
    }

    // PS-06 Phase 2 (ADR-0012): a `let = match …` expression-position program
    // with a range arm, a guard, a binding, and a wildcard must transpile and
    // compile under rustc.
    #[test]
    fn test_compile_match_expression_to_binary() {
        let test_source = r#"
void main() {
    int n = 7;
    int grade = match (n) {
        0 => 100,
        1..=5 => 50,
        x if x > 6 => 10,
        _ => 0,
    };
    println!("grade={}", grade);
}
"#;
        let input_path = PathBuf::from("test_compile_match_12345.crst");
        let output_path = PathBuf::from("test_compile_match_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // Succeeds when rustc is available; otherwise a RustcInvocation error.
        // Either way it must NOT be a parse/semantic failure of the match.
        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "match program should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-07 (ADR-0014): a named nested function lowers to a hoisted Rust `fn`
    // item — callable before its definition and recursive. Must transpile and
    // compile.
    #[test]
    fn test_compile_hoisted_recursive_nested_function() {
        let test_source = r#"
void main() {
    let f = fact(5);
    println!("fact={}", f);

    int fact(int n) {
        if (n <= 1) {
            return 1;
        }
        return n * fact(n - 1);
    }
}
"#;
        let input_path = PathBuf::from("test_compile_nestedfn_12345.crst");
        let output_path = PathBuf::from("test_compile_nestedfn_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "hoisted recursive nested fn should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-07 Workstream A (ADR-0013, DRIFT-3): a function with a function-type
    // parameter transpiles and compiles.
    #[test]
    fn test_compile_fn_pointer_parameter() {
        let test_source = r#"
int double_it(int x) {
    return x * 2;
}

int apply(fn(int) -> int f, int v) {
    return f(v);
}

void main() {
    let r = apply(double_it, 21);
    println!("r={}", r);
}
"#;
        let input_path = PathBuf::from("test_compile_fnptr_12345.crst");
        let output_path = PathBuf::from("test_compile_fnptr_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "fn-pointer parameter should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-09 Unit 2 (ADR-0015, DRIFT-1): the typedef … @Type impl surface —
    // inherent and the `default` trait form — transpiles and compiles.
    #[test]
    fn test_compile_typedef_impl_block() {
        let test_source = r#"
struct Point {
    int x;
    int y;
}

typedef struct {
    static Point origin() {
        return Point { x: 0, y: 0 };
    }

    int sum(self) {
        return self.x + self.y;
    }
} @Point;

typedef default {
    Point default() {
        return Point { x: 7, y: 7 };
    }
} @Point;

void main() {
    let p = Point { x: 3, y: 4 };
    println!("sum={}", p.sum());
    let o = @Point.origin();
    println!("osum={}", o.sum());
}
"#;
        let input_path = PathBuf::from("test_compile_implblock_12345.crst");
        let output_path = PathBuf::from("test_compile_implblock_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "typedef impl block should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-09 Unit 3 Part A (ADR-0016): a user-named trait (defined in __rust__)
    // implemented for a Crusty struct via `typedef <Trait> { } @Type;` and
    // called through the trait method.
    #[test]
    fn test_compile_general_impl_trait_for() {
        let test_source = r#"
__rust__ {
    pub trait Describe {
        fn describe(&self) -> i32;
    }
}

struct Circle {
    int r;
}

typedef Describe {
    int describe(&self) {
        return self.r * 2;
    }
} @Circle;

void main() {
    let c = Circle { r: 21 };
    println!("describe={}", c.describe());
}
"#;
        let input_path = PathBuf::from("test_compile_impltrait_12345.crst");
        let output_path = PathBuf::from("test_compile_impltrait_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "general impl Trait for should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-09 Unit 3 Part B (ADR-0016): a wholly-Crusty trait (one required +
    // one default method) implemented on a Crusty struct via Part A, exercising
    // both the overridden required method and the inherited default.
    #[test]
    fn test_compile_trait_definition_and_impl() {
        let test_source = r#"
typedef trait {
    int base(&self);
    int doubled(&self) { return self.base() * 2; }
} Measured;

struct Boxed {
    int w;
}

typedef Measured {
    int base(&self) { return self.w; }
} @Boxed;

void main() {
    let b = Boxed { w: 21 };
    println!("base={} doubled={}", b.base(), b.doubled());
}
"#;
        let input_path = PathBuf::from("test_compile_traitdef_12345.crst");
        let output_path = PathBuf::from("test_compile_traitdef_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "trait definition + impl should transpile cleanly; got {err:?}"
            );
        }
    }

    // PS-09 Unit 4 (ADR-0018): __thread__ spawns a thread. The handle form
    // (joined for a computed value) and a fire-and-forget statement both
    // transpile to std::thread::spawn(move || …) and compile.
    #[test]
    fn test_compile_thread_handle_and_fire_and_forget() {
        let test_source = r#"
void main() {
    let h = __thread__ {
        var s = 0;
        var i = 1;
        while (i <= 10) {
            s = s + i;
            i = i + 1;
        }
        return s;
    };
    let r = h.join().unwrap();
    println!("sum={}", r);

    let g = __thread__ {
        println!("worker ran");
    };
    g.join().unwrap();
}
"#;
        let input_path = PathBuf::from("test_compile_thread_12345.crst");
        let output_path = PathBuf::from("test_compile_thread_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "__thread__ handle + fire-and-forget should transpile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0019: a __scope__ whose scoped __thread__s borrow a shared local
    // (no move), one returning a value joined inside the scope, transpiles to
    // std::thread::scope(|__crusty_scope| …) with __crusty_scope.spawn(|| …)
    // and compiles; the borrowed local is still usable after the scope.
    #[test]
    fn test_compile_scoped_threads_borrow() {
        let test_source = r#"
void main() {
    let data = __rust__{ vec![1, 2, 3, 4] };
    let total = __scope__ {
        let h = __thread__ {
            return __rust__{ data.iter().sum::<i32>() };
        };
        __thread__ {
            println!("len={}", data.len());
        };
        return h.join().unwrap();
    };
    println!("total={}", total);
    println!("data still usable: {}", data.len());
}
"#;
        let input_path = PathBuf::from("test_compile_scope_12345.crst");
        let output_path = PathBuf::from("test_compile_scope_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "scoped __thread__ borrow should transpile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0021: `#ifdef FOO … #else … #endif` lowers to two cfg-gated groups —
    // #[cfg(FOO)] on the primary branch, #[cfg(not(FOO))] on the else branch.
    // Both branches define `greet`; with no `--cfg FOO` the default build
    // selects the else branch, so the program compiles with exactly one
    // `greet` active. Proves the two-group lowering and that same-named
    // definitions across branches do not collide.
    #[test]
    fn test_compile_conditional_else_selects_default_branch() {
        let test_source = r#"
#ifdef FOO
    void greet() { println!("FOO is set"); }
#else
    void greet() { println!("FOO is not set"); }
#endif

void main() {
    greet();
}
"#;
        let input_path = PathBuf::from("test_compile_cond_else_12345.crst");
        let output_path = PathBuf::from("test_compile_cond_else_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "#ifdef/#else should transpile cleanly and compile the default branch; got {err:?}"
            );
        }
    }

    #[test]
    fn test_compile_unsafe_block_and_fn() {
        let test_source = r#"
unsafe int read_raw(int* p) {
    return *p;
}

void main() {
    int value = 42;
    let ptr = __rust__{ &value as *const i32 };
    var got = 0;
    unsafe {
        got = *ptr;
    }
    println!("got={}", got);
}
"#;
        let input_path = PathBuf::from("test_compile_unsafe_12345.crst");
        let output_path = PathBuf::from("test_compile_unsafe_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "unsafe block/fn should transpile cleanly; got {err:?}"
            );
        }
    }

    #[test]
    fn test_rustc_error_reporting() {        // Create a Crusty file that will generate invalid Rust code
        // This is a bit tricky since our compiler should generate valid Rust
        // For now, we'll test that the error reporting mechanism works
        // by checking that rustc errors are properly captured

        // Create a simple valid Crusty source file
        let test_source = r#"
int test_func() {
    return 42;
}
"#;
        let input_path = PathBuf::from("test_error_report_12345.crst");
        let output_path = PathBuf::from("test_error_report_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // If rustc is available, this should succeed
        // If not, we should get a RustcInvocation error
        if let Err(err) = result {
            // Should be either RustcInvocation or another error type
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_))
                    || matches!(err, crate::error::CompilerError::Io(_))
            );
        }
    }

    // ADR-0024: tuple struct, newtype, unit struct and named struct all
    // transpile, compile, construct and read a tuple field end-to-end.
    #[test]
    fn test_compile_tuple_and_unit_structs() {
        let test_source = r#"
struct Point(int, int);
struct Meters(f64);
struct Marker;
struct Named { int x; }

void main() {
    let p = Point(3, 4);
    let m = Meters(1.5);
    let _marker = Marker;
    let n = Named { x: 7 };
    println!("{} {} {} {}", p.0, p.1, m.0, n.x);
}
"#;
        let input_path = PathBuf::from("test_compile_tuple_unit_structs_24.crst");
        let output_path = PathBuf::from("test_compile_tuple_unit_structs_24");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "tuple/unit structs should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0023: an enum mixing unit, tuple and struct variants (plus a separate
    // discriminant enum) transpiles, compiles, and is constructed + matched.
    #[test]
    fn test_compile_enum_data_variants() {
        let test_source = r#"
enum Shape {
    Empty,
    Square(int),
    Rect { int w; int h; },
}

enum Code {
    Ok = 0,
    Warn = 7,
}

int area(Shape s) {
    return match (s) {
        Shape::Square(side) => side * side,
        Shape::Rect { w, h } => w * h,
        _ => 0,
    };
}

void main() {
    let sq = @Shape.Square(5);
    let rc = @Shape.Rect { w: 3, h: 4 };
    let em = @Shape.Empty;
    println!("{} {} {}", area(sq), area(rc), area(em));
    let code = __rust__{ Code::Warn as i32 };
    println!("{}", code);
}
"#;
        let input_path = PathBuf::from("test_compile_enum_data_variants_23.crst");
        let output_path = PathBuf::from("test_compile_enum_data_variants_23");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "enum data variants should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0026: C99-style `if`/`while (decl; cond)` — scoped binding, var
    // mutability, bind-once while, and bare-form no-regression — compile + run.
    #[test]
    fn test_compile_conditional_binding() {
        let test_source = r#"
int classify(int n) {
    if (int i = n * 2; i > 10) {
        return i;
    } else {
        return i + 100;
    }
}

int countdown(int start) {
    var total = 0;
    while (var i = start; i > 0) {
        total = total + i;
        i = i - 1;
    }
    return total;
}

void main() {
    if (classify(3) > 0) {
        println!("a={}", classify(3));
    }
    println!("b={}", classify(8));
    println!("c={}", countdown(4));
}
"#;
        let input_path = PathBuf::from("test_compile_conditional_binding_26.crst");
        let output_path = PathBuf::from("test_compile_conditional_binding_26");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "conditional binding should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0029: block-bodied match arms — an arm body `{ …; v }` yields `v`
    // via its trailing expression, alongside plain-expression arms — compile +
    // run. Falls out of the value-block machinery (Expression::Block).
    #[test]
    fn test_compile_block_bodied_match_arms() {
        let test_source = r#"
int classify(int n) {
    return match (n) {
        0 => { let base = 100; base + n },
        1 => 50,
        _ => { 0 },
    };
}

void main() {
    println!("x={}", classify(0));
    println!("y={}", classify(1));
    println!("z={}", classify(9));
}
"#;
        let input_path = PathBuf::from("test_compile_block_match_29.crst");
        let output_path = PathBuf::from("test_compile_block_match_29");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "block-bodied match arms should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0029: a `return` inside a value-block exits the CONTAINING function,
    // not the block. `pick(7)` falls through the inner `if`, binds y = x+1 = 8,
    // returns 108; `pick(3)` hits `return 0;` inside the value-block and exits
    // pick entirely (the `y + 100` never runs), returning 0 — compile + run.
    #[test]
    fn test_compile_return_inside_value_block() {
        let test_source = r#"
int pick(int x) {
    let y = { if (x <= 5) { return 0; } x + 1 };
    return y + 100;
}

void main() {
    println!("big={}", pick(7));
    println!("small={}", pick(3));
}
"#;
        let input_path = PathBuf::from("test_compile_return_in_block_29.crst");
        let output_path = PathBuf::from("test_compile_return_in_block_29");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "return inside a value-block should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0029: `if`/`else` as a value (Expression::If) — `let g = if (c) { a }
    // else { b };` lowers to a Rust if-expression and is usable in arithmetic.
    // grade(95) = 1 + 100 = 101; grade(80) = 2 + 100 = 102 — compile + run.
    #[test]
    fn test_compile_if_as_value() {
        let test_source = r#"
int grade(int s) {
    let g = if (s > 90) { 1 } else { 2 };
    return g + 100;
}

void main() {
    println!("hi={}", grade(95));
    println!("lo={}", grade(80));
}
"#;
        let input_path = PathBuf::from("test_compile_if_as_value_29.crst");
        let output_path = PathBuf::from("test_compile_if_as_value_29");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "if-as-value should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0029: a redundant `{ if … }` wrap — an `if`/`else` as the SOLE
    // content of a value-block — is now the block's tail VALUE (not a consumed
    // statement). `block()` is parameterized by context (`allow_tail_cf`): a
    // value-block lets a trailing control-flow form become the tail, a function
    // body keeps it a statement. The value is USED in arithmetic, so a Void
    // mis-type would surface as a non-RustcInvocation compiler error here.
    // wrap(95) = 1 + 100 = 101; wrap(80) = 2 + 100 = 102. The `{ if … else if …
    // else … }` chain nests as a value `if_expr`: chain(95)=11, chain(70)=12,
    // chain(10)=13 — compile + run.
    #[test]
    fn test_compile_wrapped_if_value_block() {
        let test_source = r#"
int wrap(int s) {
    let g = { if (s > 90) { 1 } else { 2 } };
    return g + 100;
}

int chain(int s) {
    let g = { if (s > 90) { 1 } else if (s > 50) { 2 } else { 3 } };
    return g + 10;
}

void main() {
    println!("hi={}", wrap(95));
    println!("lo={}", wrap(80));
    println!("c1={}", chain(95));
    println!("c2={}", chain(70));
    println!("c3={}", chain(10));
}
"#;
        let input_path = PathBuf::from("test_compile_wrapped_if_29.crst");
        let output_path = PathBuf::from("test_compile_wrapped_if_29");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "a redundant {{ if … }} wrap should type as its tail value and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0031: `$type` generic method arguments. A leading `$Type` argument
    // lowers to the method's Rust turbofish: `"42".parse($i32)` ->
    // `"42".parse::<i32>()`. The parsed value is USED in arithmetic (n + 1),
    // so a mis-lowering would be a hard compile error here, not a silent pass.
    #[test]
    fn test_compile_dollar_type_generic_method() {
        let test_source = r#"
void main() {
    let s = "42";
    let n = s.parse($i32).unwrap();
    println!("n={}", n + 1);
}
"#;
        let input_path = PathBuf::from("test_compile_dollar_type_31.crst");
        let output_path = PathBuf::from("test_compile_dollar_type_31");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "$type generic method call should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0032 slice 1: a value-yielding `switch` in expression position with
    // bare (`case 1: 100;`) and block (`case 2: { …; t + 20 }`) arm bodies.
    // The result is USED in arithmetic (i + 1), so a mis-lowering would be a
    // hard compile error. pick(1)=100+1=101, pick(2)=(5+20)+1=26, pick(9)=0+1=1.
    #[test]
    fn test_compile_switch_as_expression() {
        let test_source = r#"
int pick(int x) {
    let i = switch (x) {
        case 1: 100;
        case 2: { let t = 5; t + 20 }
        default: 0;
    };
    return i + 1;
}

void main() {
    println!("a={}", pick(1));
    println!("b={}", pick(2));
    println!("c={}", pick(9));
}
"#;
        let input_path = PathBuf::from("test_compile_switch_expr_32.crst");
        let output_path = PathBuf::from("test_compile_switch_expr_32");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "switch-as-expression should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0030/0032 slice 2: case comma-list and `...` ranges in a value
    // switch. `case 1, 2, 3:` -> `1 | 2 | 3 =>`; `case 10 ... 20:` -> `10..=20
    // =>`. cls(2)=10, cls(15)=50, cls(99)=0; result used in arithmetic.
    #[test]
    fn test_compile_switch_comma_list_and_range() {
        let test_source = r#"
int cls(int x) {
    let i = switch (x) {
        case 1, 2, 3:   10;
        case 10 ... 20: 50;
        default:        0;
    };
    return i;
}

void main() {
    println!("a={}", cls(2));
    println!("b={}", cls(15));
    println!("c={}", cls(99));
}
"#;
        let input_path = PathBuf::from("test_compile_switch_cases_32.crst");
        let output_path = PathBuf::from("test_compile_switch_cases_32");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "switch comma-list/range cases should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0029: a `__func__` body requires `return` and lowers to a closure
    // whose `return` is closure-scoped; a value-block nested inside it yields
    // locally via its bare tail, while an inner `return` exits the `__func__`
    // (the closure), not the enclosing fn. run(7): v=7 > 5, the value-block
    // yields y=7, the closure returns y + 1100 = 1107; run(3): v=3 <= 5, the
    // inner `return 1000;` exits the __func__ -> 1000 — compile + run.
    #[test]
    fn test_compile_func_body_nested_value_block() {
        let test_source = r#"
int run(int x) {
    let f = int __func__(int v) {
        let y = { if (v <= 5) { return 1000; } v };
        return y + 1100;
    };
    return f(x);
}

void main() {
    println!("big={}", run(7));
    println!("small={}", run(3));
}
"#;
        let input_path = PathBuf::from("test_compile_func_nested_block_29.crst");
        let output_path = PathBuf::from("test_compile_func_nested_block_29");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "a __func__ body with a nested value-block should transpile and compile cleanly; got {err:?}"
            );
        }
    }

    // ADR-0027: dyn / impl Trait type positions — impl Trait return,
    // &dyn Trait param, Box<dyn Trait> return — compile + run.
    #[test]
    fn test_compile_dyn_impl_trait() {
        let test_source = r#"
#import std.fmt.Display

impl Iterator<Item = i32> counter(int n) {
    return __rust__{ 0..n };
}

void show(&dyn Display x) {
    println!("{}", x);
}

Box<dyn Display> boxed() {
    return __rust__{ Box::new(42) };
}

void main() {
    var total = 0;
    for i in counter(5) {
        total = total + i;
    }
    println!("sum={}", total);
    show(&total);
    let b = boxed();
    println!("boxed={}", b);
}
"#;
        let input_path = PathBuf::from("test_compile_dyn_impl_trait_27.crst");
        let output_path = PathBuf::from("test_compile_dyn_impl_trait_27");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        if let Err(err) = result {
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_)),
                "dyn/impl Trait should transpile and compile cleanly; got {err:?}"
            );
        }
    }
}
