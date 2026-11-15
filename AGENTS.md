# Forge project rules

## Writing

- Use ASD-STE100 Simplified Technical English for all writing in this chat.
- Use short sentences, active voice and consistent technical terms.
- Give one instruction per sentence. Explain technical terms when necessary.
- Keep code identifiers and quoted source text exact.

## Commit history

- Use a short imperative title that states the change.
- Put Why, What and Validation sections in each commit message.
- Keep each commit limited to one logical change.
- Apply review corrections to the commits that introduced the code.
- Use fixup commits and autosquash them before the next push.
- Do not keep a separate commit for each review response.
- Check the revised commits. Use an explicit force-with-lease value when you push.
- If the remote branch has changed, inspect it before you continue.

## Tests

- Put Rust implementation in name.rs and its unit tests in name_test.rs.
- Keep the two files in the same directory.
- Run the checks that are necessary for the change.
- Use scripts/check.sh for the full host and QEMU test suite.

See CONTRIBUTING.md for examples.
