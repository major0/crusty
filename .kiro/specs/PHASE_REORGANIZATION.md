# Crusty Compiler Phase Reorganization

## Overview

The Crusty compiler development has been reorganized into three distinct phases to better reflect the natural progression of the project and separate concerns:

- **Phase 1**: Core syntax and 1-way transpilation (Crusty → Rust)
- **Phase 2**: Tooling ecosystem (crustydoc, crustyfmt)
- **Phase 3**: Bidirectional transpilation and round-trip validation (Rust ↔ Crusty)

## Phase Definitions

### Phase 1: Core Syntax & 1-Way Transpilation

**Goal**: Establish Crusty as a working transpiler that converts Crusty syntax to Rust.

**Scope**:
- Lexer and parser for Crusty syntax
- AST representation
- Semantic analysis
- Code generation (Crusty → Rust)
- Basic CLI
- Module support via build.rs
- All core language features
- Nested functions (closures)

**Status**: Mostly complete (Tasks 1-17)

**Remaining Work**:
- Task 17.3-17.6: Complete nested function implementation
- Task 18: Checkpoint validation
- Task 19-20: VTable and module system (optional for Phase 1)
- Task 23: main() validation
- Task 24: Doc comment preservation verification
- Task 27: Additional language features (extern, __asm__, __rust__, conditional compilation)
- Task 28-30: Error improvements, pointer arithmetic, lifetime inference
- Task 31-37: Integration tests, performance, documentation, final validation

### Phase 2: Tooling Ecosystem

**Goal**: Complete the developer experience with professional documentation and formatting tools.

**Scope**:
- crustydoc: Documentation generator (leverages rustdoc)
- crustyfmt: Code formatter
- Build tool integration
- Pre-commit hooks
- CI/CD integration

**Status**: Requirements and design complete, ready for implementation

**Tasks**: See `.kiro/specs/crusty-compiler-phase2/tasks.md`

**Dependencies**: Phase 1 completion

### Phase 3: Bidirectional Transpilation

**Goal**: Prove Crusty syntax completeness through round-trip transpilation.

**Scope**:
- Rust parser integration (syn crate)
- Rust → Crusty code generation
- Round-trip validation framework
- Property-based testing for round-trip correctness
- Corpus testing with real Rust code

**Status**: Requirements and design complete, ready for implementation after Phase 2

**Tasks**: See `.kiro/specs/crusty-compiler-phase3/tasks.md`

**Dependencies**: Phase 1 and Phase 2 completion

## Tasks Moved from Phase 1

The following tasks were originally in Phase 1 but have been moved to appropriate phases:

### Moved to Phase 2:
- **Task 24**: Documentation comment preservation (verification)
- **Task 25**: crustydoc implementation
- **Task 26**: crustyfmt implementation

### Moved to Phase 3:
- **Task 21**: Rust parser integration
- **Task 22**: Crusty code generation from Rust
- **Task 32.2**: Round-trip property tests

## Phase 1 Cleanup

Phase 1 tasks file should be cleaned up to:

1. **Remove Phase 2 tasks** (24-26)
2. **Remove Phase 3 tasks** (21-22)
3. **Keep core transpilation tasks** (1-20, 23, 27-37)
4. **Update task numbering** to reflect removal
5. **Update dependencies** between tasks
6. **Update summary** with correct task counts

## Rationale for Reorganization

### Why Separate Phase 2?

**Tooling is distinct from core transpilation**:
- crustydoc and crustyfmt are separate binaries
- They reuse Phase 1 infrastructure but add new functionality
- They can be developed in parallel after Phase 1 core is stable
- They complete the "language ecosystem" rather than the "language itself"

**Benefits**:
- Clear milestone: Phase 1 = working transpiler
- Parallel development possible
- Easier to prioritize (core language first, tools second)
- Better separation of concerns

### Why Separate Phase 3?

**Bidirectional transpilation is validation, not core functionality**:
- Rust → Crusty is not needed for using Crusty
- It proves syntax completeness but doesn't add user-facing features
- It requires significant additional infrastructure (syn integration)
- It's a "nice to have" that validates design decisions

**Benefits**:
- Phase 1 + Phase 2 = complete, usable language
- Phase 3 = proof of concept and validation
- Can be deferred if needed
- Clear success criteria (round-trip tests pass)

## Implementation Order

### Recommended Sequence:

1. **Complete Phase 1 Core** (Priority: HIGH)
   - Finish nested functions (Task 17.3-17.6)
   - Complete remaining core features (Tasks 18-20, 23, 27-30)
   - Integration tests and validation (Tasks 31-37)
   - **Milestone**: Crusty → Rust transpilation works for all syntax

2. **Implement Phase 2 Tooling** (Priority: MEDIUM)
   - crustydoc for documentation generation
   - crustyfmt for code formatting
   - Build tool integration
   - **Milestone**: Professional development workflow complete

3. **Implement Phase 3 Validation** (Priority: LOW)
   - Rust parser integration
   - Crusty code generation from Rust
   - Round-trip validation
   - **Milestone**: Syntax completeness proven

## Success Criteria by Phase

### Phase 1 Success:
- ✅ All Crusty syntax parses correctly
- ✅ All Crusty code transpiles to valid Rust
- ✅ Generated Rust code compiles with rustc
- ✅ All Phase 1 tests pass (unit, integration, property)
- ✅ Example projects build and run
- ✅ Module system works with build.rs

### Phase 2 Success:
- ✅ crustydoc generates HTML documentation
- ✅ crustyfmt formats Crusty code correctly
- ✅ Both tools integrate with build system
- ✅ CI/CD enforces formatting
- ✅ All Phase 2 tests pass

### Phase 3 Success:
- ✅ Rust code parses into unified AST
- ✅ Unified AST generates Crusty code
- ✅ Round-trip tests pass (Crusty → Rust → Crusty)
- ✅ 95%+ of Rust std library transpiles to Crusty
- ✅ All Phase 3 tests pass

## File Structure

```
.kiro/specs/
├── crusty-compiler-phase1/
│   ├── requirements.md          (existing)
│   ├── design.md                (existing)
│   └── tasks.md                 (needs cleanup)
├── crusty-compiler-phase2/
│   ├── requirements.md          (✅ complete)
│   ├── design.md                (✅ complete)
│   └── tasks.md                 (✅ complete)
├── crusty-compiler-phase3/
│   ├── requirements.md          (✅ complete)
│   ├── design.md                (✅ complete)
│   └── tasks.md                 (✅ complete)
├── remove-rust-style-annotations/  (Phase 1 sub-feature)
├── typedef-type-alias-support/     (Phase 1 sub-feature)
└── PHASE_REORGANIZATION.md      (this file)
```

## Next Steps

1. **Review this reorganization** with the team
2. **Clean up Phase 1 tasks.md** to remove Phase 2/3 items
3. **Update Phase 1 requirements.md** if needed
4. **Begin Phase 2 implementation** once Phase 1 core is stable
5. **Plan Phase 3 implementation** for after Phase 2

## Notes

- **Phase 1 sub-features** (remove-rust-style-annotations, typedef-type-alias-support) remain in Phase 1
- **Nested functions** (Task 17) are part of Phase 1 core language features
- **VTable translation** (Task 19) is optional for Phase 1, could move to Phase 3
- **Module system** (Task 20) is essential for Phase 1
- **Documentation preservation** (Task 24) verification stays in Phase 1, but crustydoc tool is Phase 2

---

**Created:** January 31, 2026  
**Purpose:** Document the reorganization of Crusty compiler development into three distinct phases
