# Cleo Language Specification

This document describes the language as it is **currently implemented** in the parser.
Sections marked `TODO (not yet implemented)` are designed and documented but the parser
or compiler does not handle them yet. Sections marked `TODO` are open design questions.

---

## Index

- [Philosophy](#philosophy)
- [File Structure](#file-structure)
- [Attributes](#attributes)
  - [`@inline`](#inline)
  - [`@if`](#if)
- [Declarations](#declarations)
  - [Module-level constant](#module-level-constant)
  - [Function declaration](#function-declaration)
  - [Type declaration](#type-declaration)
  - [Import declaration](#import-declaration)
  - [Module resolution](#module-resolution)
- [Types](#types)
  - [Pointer semantics](#pointer-semantics)
  - [Pointer arithmetic](#pointer-arithmetic)
  - [Array element access](#array-element-access)
  - [Array literals](#array-literals)
  - [String literal type](#string-literal-type)
  - [Generic parameters](#generic-parameters-declaration-site)
- [Traits](#traits)
  - [Definition](#definition)
  - [Implementation](#implementation)
  - [Usage as generic bounds](#usage-as-generic-bounds)
  - [Method call syntax](#method-call-syntax)
- [Statements](#statements)
  - [Variable declaration](#variable-declaration)
  - [Defer](#defer)
  - [Return](#return)
  - [Loop](#loop)
  - [While loop](#while-loop)
  - [For loop](#for-loop)
  - [Match statement](#match-statement)
  - [Break / Continue](#break--continue)
- [Expressions](#expressions)
  - [Literals](#literals)
  - [Paths and field access](#paths-and-field-access)
  - [Struct initialization](#struct-initialization)
  - [If expression](#if-expression)
  - [Cast](#cast)
  - [Assignment](#assignment)
- [Blocks](#blocks)
- [Lexer notes](#lexer-notes)
  - [Comments](#comments)
  - [Whitespace](#whitespace)
  - [Automatic semicolon insertion](#automatic-semicolon-insertion)
  - [Tokens defined but unused by the parser](#tokens-defined-but-unused-by-the-parser)
- [Enum variant access](#enum-variant-access)
- [Macros](#macros)
- [Open design questions](#open-design-questions-todo)

---

## Philosophy

- Explicit is better than implicit.
- No hidden runtime and no automatic memory management. If something allocates, it must be freed explicitly.
- Abstractions exist at compile time, not at runtime.
- Traits are monomorphized — no vtables, no dynamic dispatch built in.
- Mutability and copying are always visible in the code.
- The execution model is simple and predictable.
- Interoperability with C is a first-class design goal.
- The language defines few primitives; higher-level behavior lives in libraries.
- Performance characteristics must be obvious from reading the source.
- This language assumes a competent programmer and does not hide consequences.
- No ownership system, no garbage collection, no reference counting, no borrow checker, no lifetime annotations.
- Name resolution is performed at module scope before type checking.
- Undefined behavior exists and follows the C model: dereferencing invalid pointers, data races, and violating alignment rules result in UB.

---

## File Structure

A Cleo source file is a **compilation unit** (module). It consists of a sequence of
top-level declarations. There is no required entry point order — declarations may appear
in any order within the file.

```
unit = decl*
```

---

## Attributes

Attributes are compile-time annotations written on their own line, prefixed with `@`.
Each attribute applies to the immediately following item. Multiple attributes stack —
they all apply to the next non-attribute item, in top-to-bottom order.

```
attr      = "@" ident ["(" attr_arg ("," attr_arg)* ")"]
attr_arg  = ident "=" ident | ident
attributed = attr* (fn_decl | type_decl | const_decl | stmt)
```

```
@if(platform = windows)
@inline
fn platform_init() { ... }
```

Attributes are evaluated entirely at compile time. An unknown attribute name is an error.

**TODO (not yet implemented):** The parser does not yet handle the `@` token. All
attribute syntax described here is designed but not implemented.

### `@inline`

Valid on: function declarations only.

Marks the function for inlining at call sites.

```
@inline
fn add(a: i32, b: i32) i32 { return a + b }
```

**TODO (not yet implemented):** The parser does not yet handle `@inline`. Inlining
cannot be requested until attribute parsing is implemented.

### `@if`

Valid on: any declaration or statement.

Conditionally includes the decorated item at compile time. If the condition is false
the item is completely absent from the output — no code is emitted.

```
attr_cond = ident "=" ident   // key = value
```

```
@if(platform = windows)
fn get_handle() *void { ... }

@if(platform = linux)
fn get_handle() *void { ... }
```

Inside a function body `@if` applies to the next statement:

```
fn init() {
    @if(platform = windows)
    win32_init()

    @if(platform = linux)
    linux_init()
}
```

#### Condition keys

All conditions are **externally provided** — the source code never defines flags.
There are two sources:

| Source             | Example key  | Values                                   |
|--------------------|--------------|------------------------------------------|
| Compiler target    | `platform`   | `windows`, `linux`, `macos`, `wasm`, ... |
| Compiler target    | `arch`       | `x86_64`, `arm64`, `x86`, `arm`, ...     |
| Build flag (`-D`)  | any name     | any bareword, or boolean if no value     |

Build flags are passed by the build system (or manually) at compile time:

```
cleoc main.cleo -D debug -D profile=release
```

```
@if(debug)
fn dump_state() { ... }

@if(profile = release)
fn assert_disabled() { ... }
```

There is no way to define or set a flag from inside a `.cleo` source file.
Flags are purely a build-time concern, controlled by the project tooling.

**TODO:** Exact flag name for the compiler flag (`-D`, `-C`, other) — TBD.

---

## Declarations

All top-level declarations can be prefixed with `pub` to make them visible to other modules.

```
decl     = ["pub"] (fn_decl | type_decl | import_decl | const_decl)
```

### Module-level constant

**TODO (not yet implemented):** `const` at module scope is not yet parsed as a
top-level declaration. Currently `const` only works inside function bodies via the
`local` rule. A module-level const needs its own `Decl` variant.

```
const PI: f32 = 3.14159
const MAX_SIZE: i32 = 1024
```

### Function declaration

```
fn_decl = "fn" ident [generic_params] fn_params [type] block
```

- Generic parameters are declared with `[T, U: Bound, ...]` (square brackets, not angle brackets).
- The return type is optional and appears **directly after the parameter list**, with no `->`.
- If return type is omitted the function returns nothing (void).
- Inlining is requested with the `@inline` attribute (see [Attributes](#attributes)).

```
fn add(a: i32, b: i32) i32 { return a + b }
fn log[T](value: T) { }

@inline
fn clamp(v: i32, lo: i32, hi: i32) i32 { ... }
```

`self` is a **hard keyword**. It can only appear as the first parameter of a function
declaration. Using it in any other parameter position is a parse error.

```
fn draw(self: *Shape) i32 { ... }       // valid: self is first
fn bad(a: i32, self: *Shape) { ... }    // parse error: self not in first position
```

The semantic rule that a function with `self` as first param is a method of the
receiver type is enforced by the analyzer.

`self` is a distinct token from `Ident` in the lexer, not a contextual keyword —
this is what makes the first-position rule a parse error rather than a semantic
check: `self` in any parameter position other than first simply matches nothing,
since `fn_param` only accepts `Ident`.

Inside the function body, `self` is also valid as the leading segment of a path
expression, enabling field and method access:

```
fn draw(self: *Shape) {
    var a = self.area   // self as first path segment
    self.render()
}
```

`self` cannot appear after a `.` — only as the first segment.

### Type declaration

```
type_decl = "type" ident [generic_params] "=" type_body

type_body = struct_decl
          | enum_decl
          | trait_decl
          | type          (alias)
```

**Struct:**

```
struct_decl = "struct" "{" [field ("," field)* [","]] "}"
field       = ident ":" type
```

```
type Point[T] = struct {
    x: T,
    y: T,
}
```

**Enum:**

```
enum_decl   = "enum" "{" [ident ("," ident)* [","]] "}"
```

Variants are plain identifiers with no associated data.

```
type Day = enum { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
```

**TODO:** Enum variants with associated data (tagged unions) are planned but not defined.

**Trait:**

```
trait_decl   = "trait" "{" [trait_method (";" trait_method)* [";"]] "}"
trait_method = "fn" fn_signature
fn_signature = ident [generic_params] fn_params [type]
```

A trait body contains only function _signatures_ (no bodies).

```
type Printable = trait {
    fn print(self: *Self)
    fn size(self: *Self) i32
}
```

**TODO:** `Self` inside a trait is parsed as a plain identifier. The compiler must
resolve it to the implementing type during semantic analysis.

**Alias:**
Any type expression on the right-hand side that is not `struct`, `enum`, or `trait`
is treated as a type alias.

```
type Meters = f32
type IntPtr = *i32
```

### Import declaration

```
import_decl = "import" ident ("." ident)*
```

```
import math
import std.collections.HashMap
```

### Module resolution

Every `.cleo` file is a module. There are no implicit wildcard imports and no
folder-level entry points — every import must resolve to a specific file.

#### Search paths

The compiler accepts two sources of search roots, in priority order:

1. **Project root** — the directory of the entry file passed to the compiler.
   `cleoc main.cleo` makes the directory containing `main.cleo` the project root.
2. **`-I` flags** — additional search roots for external libraries, one per flag.
   Searched in the order they are given, after the project root.

```
cleoc main.cleo -I /usr/local/cleo/std -I /home/user/.cleo/libs
```

Project management tooling (separate from the compiler) is responsible for reading
a manifest file and generating the appropriate `-I` flags. The compiler itself has
no knowledge of package managers, registries, or lock files.

#### Relative imports

**TODO (not yet implemented):** The parser does not yet handle a leading `.`.
The `ImportDecl` struct needs a `relative: bool` field and the parser needs to
detect the leading `Dot` token before the first identifier.

A leading `.` makes an import relative to the **current file's directory**, regardless
of the project root or `-I` paths.

```
import .utils        // same directory as the current file: ./utils.cleo
import .helpers.fmt  // ./helpers.cleo, item fmt  (or ./helpers/fmt.cleo)
```

#### Absolute imports

**TODO (not yet implemented):** The compiler does not yet perform module resolution.
The parser produces an `ImportDecl` with the raw path segments, but no file lookup,
`-I` flag handling, or namespace/item distinction is implemented.

An import with no leading `.` is resolved against each search root in order
(project root first, then `-I` paths) until a match is found.

The resolution algorithm for each search root is:

1. **Try the full path as a file.** Join all segments with `/` and append `.cleo`.
   If the file exists, this is a **namespace import** — all `pub` items of that
   module become accessible under the module name as a prefix.
2. **Try splitting the last segment as an item.** If the full-path file does not
   exist, treat all segments except the last as the file path and the last segment
   as a specific exported item. If that file exists and exports that item, this is
   an **item import** — the item enters the current scope directly, without prefix.
3. Try the next search root. If all roots are exhausted, **error**.

```
import math              // math.cleo — namespace import
                         // usage: math.add(...), var v: math.Vector2

import math.Vector2      // (1) tries math/Vector2.cleo
                         // (2) if not found: math.cleo + item Vector2
                         // usage: Vector2 (no prefix)

import shapes.circle.Triangle
                         // (1) tries shapes/circle/Triangle.cleo
                         // (2) if not found: shapes/circle.cleo + item Triangle
                         // usage: Triangle (no prefix)
```

There is no `import math.*` — to get unqualified access to multiple items, import
each one explicitly.

**TODO:** `pub import` for re-exporting is lexed (`pub` is a token) but not yet defined.

---

## Types

```
type     = array_prefix ptr_prefix path
array_prefix = ("[" integer "]")*
ptr_prefix   = ("*" | "*" "const")*
```

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

**TODO:** `void` as an explicit return type is not parsed. Currently omitting the
return type means void; there is no way to write `void` explicitly.

**TODO:** Function pointer types are not defined.

### Pointer semantics

`*T` grants permission to mutate through the pointer.
`*const T` forbids mutation through the pointer — it does **not** mean the underlying
data is immutable, only that this particular pointer cannot be used to write to it.

The language does not assume any aliasing guarantees. Multiple mutable pointers may
alias the same memory location. The compiler performs no alias-based optimizations
unless explicitly proven safe.

Implicit coercion from `*T` to `*const T` is allowed (upgrading to const).
The reverse requires an explicit cast.

```
var x: i32 = 10
var p: *i32 = &x
var cp: *const i32 = p        // ok: implicit upgrade
var p2: *i32 = cp             // error: cannot downgrade const pointer implicitly
var p3: *i32 = cp as *i32     // ok: explicit cast
```

### Pointer arithmetic

Adding or subtracting an integer to a pointer advances it by that many elements
of the pointed-to type (same semantics as C).

```
var arr: [10]i32 = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
var ptr: *const i32 = &arr
var fifth: *const i32 = ptr + 4   // points to arr[4]
var val: i32 = *fifth             // 4
```

Pointer field access uses `.` — the same dot operator used for value access. The compiler
auto-dereferences the pointer at the access site. There is no `->` operator.

```
var p: *Point = &my_point
p.x = 10          // equivalent to (*p).x = 10
var n: i32 = p.x  // equivalent to (*p).x
```

### Array element access

There is no `[index]` subscript operator for arrays. The square-bracket syntax is reserved
for generics (`Vec[i32]`) and array type declarations (`[N]T`). To access an element, take
a pointer to the array and use pointer arithmetic:

```
var arr: [5]i32 = {1, 2, 3, 4, 5}
var ptr: *const i32 = &arr
var third: i32 = *(ptr + 2)   // arr[2] = 3
```

The standard library provides bounds-checked helpers (`at`, `get`) that wrap this pattern.

### Array literals

**TODO (not yet implemented):** Array literal syntax `{expr, expr, ...}` is not yet
parsed. Intended to create a fixed-size array inline, assignable to `[N]T`.

```
var arr: [5]i32 = {1, 2, 3, 4, 5}
```

### String literal type

String literals have type `const [N]u8` where `N` is the number of characters
(not counting the null terminator). The compiler stores them in static memory
with a null terminator appended — the null byte is not part of the type but is
accessible via pointer arithmetic.

```
var s: [5]u8 = "hello"          // type: const [5]u8, 6 bytes in memory
var p: *const u8 = &s           // compatible with C string functions
```

No built-in string type exists. Strings are `[N]u8` arrays or `*const u8` pointers.

### Generic parameters (declaration site)

```
generic_params = "[" generic_param ("," generic_param)* [","] "]"
generic_param  = ident [":" type]
```

- The bound after `:` is a type (usually a trait name, but the parser accepts any type).
- Bounds are single — there is no syntax for multiple bounds on one parameter.

**TODO:** Multiple bounds per generic parameter (e.g. `T: Printable + Comparable`)
are not yet defined.

---

## Traits

Traits define a set of method signatures used as compile-time constraints.
They are a **structural** typing mechanism — a type satisfies a trait if it has
the required methods, with no explicit declaration of conformance needed.

There are no `impl` blocks and no `for TraitName` annotations. The check happens
at the monomorphization site, not at the definition site.

### Definition

Traits are declared as a type body (see [Type declaration](#type-declaration)):

```
type Printable = trait {
    fn print(self: *Self)
}

type Comparable = trait {
    fn less(self: *Self, other: *Self) bool
    fn equal(self: *Self, other: *Self) bool
}
```

`Self` is a placeholder that the compiler replaces with the concrete type being
checked against the trait. It has no runtime representation.

**TODO:** `Self` is currently parsed as a plain identifier. Semantic resolution to the
concrete type is not yet implemented.

### Satisfaction (structural check)

A type `T` satisfies a trait if, for every signature in the trait, there exists a
free function in scope with a matching name and parameter types (with `Self` replaced
by `T`). No annotation on the function is required.

```
type Name = struct { first: *u8, last: *u8 }

fn print(self: *Name) {
    // no "for Printable" needed — Name satisfies Printable structurally
}
```

The check is performed when the type is used as a generic argument with a trait bound.
If the type is missing any required method the compiler errors at the call site:

```
fn log[T: Printable](value: *T) { value.print() }

log(&my_name)   // ok: Name has print(self: *Name)
log(&my_point)  // error: Point does not have print(self: *Point)
```

### Usage as generic bounds

A trait name after `:` in a generic parameter constrains which types can be used:

```
fn log[T: Printable](value: *T) {
    value.print()
}

fn max[T: Comparable](a: *T, b: *T) *T {
    if a.less(b) { return b }
    return a
}
```

The compiler monomorphizes the function for each concrete type `T` is instantiated with
and verifies trait satisfaction for that type at the call site.

**TODO:** Multiple bounds per parameter (`T: Printable + Comparable`) — syntax not yet defined.

### Method call syntax

`value.method(args)` is syntactic sugar for a free function call with the receiver
as the first argument:

```
value.print()     // desugars to: print(&value)   when value is not a pointer
ptr.print()       // desugars to: print(ptr)
```

The `.` operator auto-dereferences — calling a method on a pointer and on a value
uses the same syntax.

**TODO:** Exact auto-ref rules (when `&value` is inserted vs passed directly) are not yet defined.

---

## Statements

Statements inside a block are separated by semicolons. A trailing semicolon is not
required after the last statement.

```
stmt = local | defer | return | break | continue | loop | expr
```

### Variable declaration

```
local = ("var" | "const") ident [":" type] ["=" expr]
```

- `var` declares a mutable binding.
- `const` declares an immutable binding. The value does not have to be a compile-time
  constant at the language level — that distinction is enforced by the analyzer.
- The type annotation is optional when an initializer is present (type inference).
- Both the type and the initializer can be omitted together (declaration without init).

```
var x: i32 = 10
var y: i32
const N: i32 = 100
const msg = "hello"
```

**Note:** `let` does not exist. The language only has `var` and `const`.

### Defer

```
defer = "defer" expr
```

Schedules `expr` to execute when the enclosing scope exits.
Deferred expressions execute in reverse declaration order.

```
defer free(ptr)
defer close(fd)
```

Once blocks are expressions (see blocks section), `defer { ... }` works automatically
through `defer expr` — no separate handling needed.

### Return

```
return_stmt = "return" [expr]
```

`return` exits the **enclosing function** and optionally produces its value. It never
exits a block expression — use `break` for that.

### Loop

```
loop_expr = "loop" block
```

An unconditional infinite loop. `loop` is an **expression**: it evaluates to the value
carried by the `break` that exits it.

```
var a: i32 = loop {
    break 40
}
```

If `break` carries no value, the loop expression yields void; using it where a value
is expected is a compile-time error.

**TODO (not yet implemented):** `loop` is currently parsed as a statement, not as an
expression. It cannot appear on the right-hand side of an assignment until blocks are
expressions and `break <value>` is implemented.

**TODO:** `while` and `for` loops. The tokens exist in the lexer but no parser rule
handles them.

### While loop

**TODO (not yet implemented):** The `while` token exists in the lexer but the parser
has no rule for it.

```
while_stmt = "while" expr block
```

### For loop

**TODO (not yet implemented):** The `for` token exists in the lexer but the parser
has no rule for it. The iteration syntax is not yet defined.

### Match statement

**TODO (not yet implemented):** The `match` token exists in the lexer but the parser
has no rule for it.

Match is an exhaustive switch with pattern matching. Every possible value must be
covered, either explicitly or with a wildcard `_` arm.

```
match today {
    Day.Sunday, Day.Saturday => {
        println("It's the weekend!")
    },
    _ => {
        println("It's a weekday.")
    }
}
```

- Multiple patterns in a single arm are separated by `,`.
- Arms use `=>` (fat arrow).
- `_` is the wildcard that matches anything.
- The compiler must verify exhaustiveness.

**TODO:** Pattern syntax beyond enum variant paths (e.g. struct destructuring, guards) is not yet defined.

### Break / Continue

```
break_stmt = "break" [expr]
```

`break` exits the **innermost enclosing block or loop** and optionally yields a value:

- `break` — exits without a value; the block/loop expression yields void.
- `break <expr>` — exits and the block/loop expression evaluates to `<expr>`.

```
// block expression with a value
var x: i32 = {
    var a = compute()
    break a + 1
}

// loop expression with a value
var result: i32 = loop {
    if done() { break 42 }
}
```

A block or loop without `break <value>` yields void. Using such an expression where a
typed value is expected is a compile-time error — there is no implicit last-expression value.

`continue` has no value and skips to the next iteration of the innermost loop.

**TODO (not yet implemented):** `break <value>` is not yet parsed. The parser currently
only handles plain `break` with no operand. Implementing this requires blocks and loops
to be expressions.

**TODO:** Labeled break/continue is not defined.

---

## Expressions

Operator precedence from lowest to highest:

| Level | Operators                         | Associativity |
| ----- | --------------------------------- | ------------- |
| 1     | `=` `+=` `-=` `*=` `/=` (assign)  | right         |
| 2     | `\|\|`                            | left          |
| 3     | `&&`                              | left          |
| 4     | `==` `!=` `<` `>` `<=` `>=`       | none (single) |
| 5     | `+` `-`                           | left          |
| 6     | `*` `/`                           | left          |
| 7     | unary `*` `&` `-` `!`             | prefix        |
| 8     | `as` (cast)                       | postfix       |
| 9     | `.field` `(args)` (access / call) | left          |

**TODO:** `%` (modulo) is a lexed token but has no parser rule.

**TODO:** Bitwise operators `&`, `|`, `^`, `<<`, `>>`, `~` are lexed but not parsed.

**TODO:** `++` and `--` are lexed but not parsed.

### Literals

| Syntax           | Type                                                  |
| ---------------- | ----------------------------------------------------- |
| `42`             | integer                                               |
| `42i32`          | integer with suffix (`i8/i16/i32/i64/u8/u16/u32/u64`) |
| `0xFF`           | hex integer                                           |
| `0b1010`         | binary integer                                        |
| `3.14`           | float                                                 |
| `"hello"`        | string                                                |
| `true` / `false` | bool                                                  |

**TODO:** `null` is a lexed keyword but is not handled as a literal in the expression
parser.

**TODO:** Float suffixes (`f32`, `f64`) from the C spec are not yet handled in the lexer.

**TODO:** Integer suffixes `u/U`, `l/L`, `ul/UL`, `ll/LL` from standard C are not
handled. Only the Cleo-specific suffixes (`i8`..`u64`) are lexed.

**TODO:** String escape sequences are not validated by the lexer (the raw bytes are
passed through as-is).

### Paths and field access

A path is a dot-separated sequence of segments, each optionally followed by generic
arguments in square brackets.

```
path    = segment ("." segment)*
segment = ident ["[" type ("," type)* [","] "]"]
```

```
x
module.function
Point[i32]
std.Map[String, i32].new
```

The same syntax is used for:

- Variable references (`x`)
- Module-qualified names (`math.add`)
- Generic instantiations (`Vec[i32]`)
- Method chains (`obj.method`)

The parser produces a single `PathExpr` node; disambiguation happens in the analyzer.

### Struct initialization

```
struct_init = path "{" ["." ident "=" expr ("," "." ident "=" expr)* [","]] "}"
```

```
Point { .x = 1, .y = 2 }
Pair[i32, f32] { .first = 10, .second = 3.14 }
```

### If expression

```
if_expr = "if" expr block ["else" (if_expr | block)]
```

`if` is an expression — it can appear anywhere an expression is expected.

**TODO (not yet implemented):** `else if` chaining is designed (the `if_expr | block`
alternative above) but not parsed. The parser currently only accepts a plain `else`
followed by a `block` — a second `if` after `else` is a parse error. Fixing this
requires the `else` branch to accept `if_expr` recursively, same as `block`.

**TODO (not yet implemented):** `if` cannot actually produce a value yet. It parses
as an expression, but its branches are `block`, and blocks have no way to yield a
value until block-as-expression and `break <value>` (see [Blocks](#blocks) and
[Break / Continue](#break--continue)) are implemented. `block`, `loop`, and `if`/`else`
becoming value-producing expressions is one connected piece of work — `while`/`for`
are intentionally out of scope for this, since they don't need to yield a value.

### Cast

```
cast = factor ["as" type]
```

`factor` is the access/call precedence level (see the precedence table below) —
`as` binds tighter than any binary operator, so `a + b as i32` casts only `b`.

```
value as i32
ptr as *const u8
```

### Assignment

```
assign = logic_or [("=" | "+=" | "-=" | "*=" | "/=") expr]
```

The left-hand side is restricted to the `logic_or` precedence level (to avoid left
recursion); the right-hand side is a full `expr`, which is what makes assignment
right-associative (`a = b = c` parses as `a = (b = c)`).

Beyond that syntactic restriction, the left-hand side must be an lvalue; this is
enforced by the analyzer, not the parser.

**TODO:** `%=` is a lexed token but not parsed as an assignment operator.

---

## Blocks

```
block = "{" [stmt (";" stmt)* [";"]] "}"
```

Statements are separated by semicolons. A trailing semicolon is optional.
An empty block `{}` is valid.

A block is an **expression**. Its value is produced explicitly by a `break <expr>`
statement inside it. There is no implicit last-expression value — a block without
`break <value>` yields void, and using it where a value is expected is a compile-time error.

```
var x: i32 = {
    var a = compute()
    break a + 1       // the block expression evaluates to a + 1
}
```

`return` inside a block exits the **enclosing function**, not the block.
To exit the block and produce a value, use `break`.

**TODO (not yet implemented):** Blocks are not yet expressions. A bare `{ ... }` is
not valid as a primary expression — only `(expr)` with parentheses is. Adding
`Block` as a variant of `Expr` and including it in `primary` would enable:

- `defer { ... }` (currently `defer` only accepts a single expression)
- `var x = { ...; break value }` (block initializers)
- Any place that accepts an expression accepting a block

---

## Lexer notes

### Comments

Single-line comments only: `// text`. No block comments.

**TODO:** Block comments (`/* */`) are not supported.

### Whitespace

Spaces, tabs, and carriage returns are skipped.

### Automatic semicolon insertion

Newlines are significant. The lexer converts a newline into a synthetic `;` token
when both of the following are true:

1. The last emitted token can end a statement:
   `Ident`, any literal (`Integer`, `Float`, `String`, `Bool`), `break`, `return`,
   `continue`, `++`, `--`, `)`, `]`, `}`
2. The next token is **not** a continuation operator:
   `+`, `-`, `*`, `/`, `%`, `.`, `)`, `}`, `]`

This allows expressions to span multiple lines naturally:

```
var x = 1 + 2    // newline → ';' inserted after 2
    + 3          // no ';' before '+', so this continues the expression above

fn add(a: i32,   // no ';' — next line starts a continuation
       b: i32) i32 {
    return a + b // newline → ';' inserted after b
}
```

Newlines that do not meet both conditions are silently discarded.

### Tokens defined but unused by the parser

The following tokens are recognized by the lexer but have no corresponding parser rule:

| Token                      | Notes                                                                |
| -------------------------- | -------------------------------------------------------------------- |
| `while`                    | loop construct, not yet parsed                                       |
| `for`                      | loop construct, not yet parsed                                       |
| `match`                    | pattern matching, not yet parsed                                     |
| `null`                     | null literal, not yet parsed as an expr                              |
| `%`                        | modulo operator                                                      |
| `%=`                       | modulo-assign operator                                               |
| `++`                       | increment                                                            |
| `--`                       | decrement                                                            |
| `&` `\|` `^` `<<` `>>` `~` | bitwise operators                                                    |
| `->`                       | recognized by the lexer, not used — Cleo uses `.` for pointer access |
| `=>`                       | intended for match arms                                              |
| `@`                        | attribute prefix — syntax designed, parser not yet implemented       |
| `#` `$` `?` `...`          | reserved symbols, purpose TBD                                        |

---

## Enum variant access

Enum variants are accessed using the type name as a namespace, unlike standard C enums.

```
type Day = enum { Monday, Tuesday, Wednesday }

var today: Day = Day.Wednesday   // qualified access required
```

This is enforced by the analyzer — the bare name `Wednesday` is not in scope.

**TODO:** How does `match` pattern matching interact with enum variant paths?

---

## Macros

**TODO (not yet implemented):** A macro system is planned but not yet designed in detail.
Macros would be defined using the language itself and operate on the AST.

Intended syntax from design docs (subject to change):

```
macro log(arg: AstNode) AstNode {
    // generates code to log arg
}

#log(x)   // macro invocation
```

The `#` token is reserved in the lexer for this purpose.
A special `AstTokenStream` node type would allow macros to receive unparsed token
streams, enabling DSL use cases.

**TODO:** Macro syntax, hygiene rules, and the AstNode API are entirely undefined.

---

## Open design questions (TODO)

- **Method lookup scope:** when checking trait satisfaction, which free functions are in scope — current module only, or also imported modules?
- **`pub import` re-exports:** semantics not yet defined.
- **Enum variants with data:** tagged unions, layout, and pattern matching syntax.
- **`while` and `for` loops:** syntax and semantics.
- **Block return values:** once `{ ... }` is a valid expression, `defer { ... }`, block initializers (`var x = { return 42 }`), and `if`/`match` as expressions work automatically.
- **`void` as an explicit type:** can you write `fn foo() void`?
- **Function pointer types:** syntax for storing and calling function pointers.
- **Multiple trait bounds:** syntax for `T: A + B`.
- **`Self` in traits:** semantic resolution of `Self` (uppercase, plain identifier) to the implementing type.
- **Labeled break/continue:** needed for nested loops.
- **`null` literal:** type, assignability rules.
- **Macros:** syntax, hygiene, AstNode API — may be deferred in favor of the attribute system for most use cases.
- **Build flag name:** exact compiler flag for passing cfg values (`-D`, `-C`, other) — TBD.
- **User-defined attributes:** custom `@myattr` beyond the built-in set — requires a definition mechanism.
- **`const` array size expressions:** `const arr: [size]i32` where `size` is a compile-time constant — requires comptime evaluation.
