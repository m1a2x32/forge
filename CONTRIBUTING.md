# Development conventions

Keep changes small enough to explain and review as one logical step. Explain the
kernel concept and hardware assumptions before extending the implementation,
then state what testing established and what remains uncertain.

## Commits

Use a concise imperative title that names the change. Split independent concerns
into separate commits and keep intermediate commits buildable where practical.
Include a structured body:

```text
Add platform-configured frame sizes

Why:
Explain the problem or constraint motivating the change.

What:
Describe the implementation and relevant design choices.

Validation:
State the checks actually run and their results.
```

Apply review corrections to the logical commits that introduced the code. Use
fixup commits and autosquash them before the next push. Do not keep a separate
commit for each review response. Update the commit message if its scope changes.
Check the revised history, then push with an explicit force-with-lease value.
If the remote branch has changed, inspect it before you continue.

## Tests

Keep Rust implementation in `name.rs` and unit tests in the sibling
`name_test.rs`. For example, `memory.rs` loads `memory_test.rs` using:

```rust
#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;
```

Keep tests with the behavior they verify in the same logical commit. Emulator
integration checks live under `scripts/`. Run the checks appropriate to the change;
`./scripts/check.sh` runs the full host and QEMU suite.

## Chat writing

Use ASD-STE100 Simplified Technical English for all writing in this chat.
Use short sentences, active voice and consistent technical terms. Give one
instruction per sentence. Explain a technical term when the reader needs it.
Keep code identifiers and quoted source text exact.
