# mp4-atom

Pure-Rust low-level ISO Base Media File Format (ISO/IEC 14496-12) atom encoder/decoder.

## Commands

```bash
just check   # check, clippy (-D warnings), fmt, shear across all features/targets
just test    # run unit and codec integration tests
just fix     # auto-fix fmt, clippy, shear
just msrv    # check and test on Rust 1.88, the declared rust-version
just i686    # test on 32-bit, where u64 sizes truncate to usize
```

Nothing runs on pull requests: run `just check` and `just test` before every push, plus `just msrv` or `just i686` when a change could break them.

## Rules

- Keep atom layout faithful to ISO/IEC 14496-12; malformed or deeply nested inputs must return `Result::Err`, never panic or exhaust memory/stack.
- Exercise large-atom streaming through `Header::read_from()` rather than buffering whole payloads when decoding incrementally.
- Add real-bitstream or round-trip tests under `src/test/` for new codecs or atoms.

Quests: before any work, run `quest context` and follow it, unless this session already holds its rules.
