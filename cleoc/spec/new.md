# Cleo Language Specification

This document describes the language.

---

# Types

Types are written **prefix-first**: arrays and pointers wrap the base type on the left.

| Syntax         | Meaning                      |
| -------------- | ---------------------------- |
| `i32`          | 32-bit signed integer        |
| `*i32`         | mutable pointer to i32       |
| `*const i32`   | immutable pointer to i32     |
| `[10]i32`      | fixed-size array of 10 i32   |
| `[4]*u8`       | fixed-size array of 4 `*u8`  |
| `Point[i32]`   | generic instantiation        |
| `std.Map[K,V]` | qualified path with generics |

**Primitive types** (resolved by name, no special syntax):
`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`

# Void\* Alternative

Instead of using `void*` for generic pointers, Cleo allows to define a empty type which can be used without interpretation. This is useful for generic data structures and APIs.

```cleo
type Any
```
