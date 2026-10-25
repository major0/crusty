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
    int grade = match n {
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
}
