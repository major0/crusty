# Concurrency

Crusty's concurrency primitive is the `__thread__` block (ADR-0017/ADR-0018). It
spawns its body on a new OS thread, lowering to `std::thread::spawn` — so it
needs **no runtime dependency**. Async Rust is reached through the `__rust__`
escape hatch rather than a first-class `async`/`await` surface.

## `__thread__`

`__thread__ { <body> }` spawns the body on a new thread. It is a **value**
whose value is the spawn's join handle, so one surface covers both uses:

### Handle form — keep the handle, join for the result

```c
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
println!("sum={}", r);     // sum=55
```

Binding the `__thread__` value gives you the join handle. `join()` is an
ordinary method call — call it (with `.unwrap()` / `?` as you like) to wait for
the thread and read its return value. The thread's return value is the body's
`return` value.

### Fire-and-forget — drop the handle

```c
__thread__ {
    log_event();
};
```

Used as a bare statement, the handle is dropped and the thread runs
independently. (If you need its output to be deterministic in a short program,
keep the handle and `join()` it.)

## Lowering

```
__thread__ { body }   →   std::thread::spawn(move || { body })
```

The closure is always `move`: the thread may outlive the enclosing function, so
it takes ownership of what it uses.

## The capture model — hand the work its data

Because the closure is `move`, an **owned** local used in the body is *moved*
into the thread:

```c
let data = make_data();
let h = __thread__ {
    return process(data);   // `data` is moved into the thread
};
```

Two consequences, both enforced by `rustc` (Crusty does not pre-check them):

- **Borrowing a local across the boundary is a compile error.** A thread that
  merely *reads* an enclosing local it cannot take ownership of fails to
  compile (rustc `E0373`). Move it in, clone it, or share it explicitly.
- **A non-`Send` value cannot cross the boundary.** Moving something like an
  `Rc` into a thread is a compile error (rustc `E0277`). Use `Arc` for shared
  ownership across threads.

## Sharing data back

`__thread__` moves data *in*; to share state *across* threads or get results
*back* beyond a single `join()`, use the standard Rust tools explicitly — an
`Arc<Mutex<…>>`, or a channel — constructed via normal calls or a `__rust__`
value. Crusty deliberately does not wrap these: the sharing model is Rust's, in
the open, not hidden behind the block.

## Async

Crusty has no first-class `async`/`await`. Write async Rust in a `__rust__`
block and call it from Crusty:

```c
__rust__ {
    pub async fn fetch(url: &str) -> String { /* … */ String::new() }
}
```

This keeps the language C-like and keeps the choice of async runtime (tokio,
async-std, …) with the application, where it belongs, rather than baking one
into the transpiler. `__thread__` is the native primitive for the common
"run this concurrently" need; reach for async only when you specifically need
it, through `__rust__`.

## Scoped threads: `__scope__`

`__thread__` moves data in; when you want several threads to **borrow** a shared
local and all finish before it is released, use `__scope__` (ADR-0019):

```c
let data = __rust__{ vec![1, 2, 3, 4] };
let total = __scope__ {
    // Scoped threads BORROW `data` (no move), auto-joined at scope end.
    let h = __thread__ { return __rust__{ data.iter().sum::<i32>() }; };
    __thread__ { println!("len={}", data.len()); };
    return h.join().unwrap();
};
println!("total={}", total);
println!("data still usable: {}", data.len());   // borrow ended — `data` is fine
```

- `__scope__ { … }` is a value-producing expression; its value is the scope
  body's returned value.
- A `__thread__` **inside** a `__scope__` is a **scoped** spawn: it **borrows**
  (no `move`) and is **auto-joined** when the scope ends. A `__thread__`
  **outside** any `__scope__` keeps its top-level `move` behaviour.
- The scope guarantees every thread finishes before the borrowed data is
  released, which is why `rustc` accepts the borrow — no `Arc`, no clone.

Lowering:
```
__scope__ { body }                       →  std::thread::scope(|__crusty_scope| { body })
__thread__ { body }   (inside __scope__)  →  __crusty_scope.spawn(|| { body })
```

The scope handle `__crusty_scope` is generated and implicit — you never write
it. A `__thread__` binds to the nearest enclosing `__scope__`.

