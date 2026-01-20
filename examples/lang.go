```
Language Philosophy
Explicit is better than implicit.
There is no hidden runtime and no automatic memory management.
If something allocates, it must be freed explicitly.
Abstractions exist at compile time, not at runtime.
Traits are monomorphized; dynamic dispatch is not built in.
Mutability, copying, and lifetimes are always visible in the code.
The execution model is simple and predictable.
Interoperability with C is a first-class design goal.
The language defines few primitives; higher-level behavior lives in libraries.
Performance characteristics must be obvious from reading the source.
Discipline is preferred over automation.
This language assumes a competent programmer and does not hide consequences.
Name resolution is performed at module scope before type checking.
The language aims to reimagine C with modern type system consistency, while preserving its explicit and predictable execution model.

The language does not assume any aliasing guarantees by default.
Multiple mutable pointers may alias the same memory location.
The compiler performs no alias-based optimizations unless explicitly proven safe.

`*T` implies permission to mutate through the pointer.
`*const T` forbids mutation through the pointer, not mutation of the underlying data.

Traits do not imply vtables or runtime representation.
All trait methods are statically resolved and monomorphized.

Enum layout is stable and ABI-compatible when exported.
The tag type and layout are defined by the compiler.

Undefined behavior exists and follows the C model.
Dereferencing invalid pointers, data races, and violating alignment rules result in undefined behavior.

No ownership system like Rust.
No garbage collection.
No reference counting.
No borrow checker.
No lifetime annotations.

Arrays: [S]T

Pointers: *T or *const T

The language does not define slices as a built-in concept.

`self` is a keyword representing the instance in trait methods this keyword determines if a functions is a method or a free function

Defer Statement: defer BODY or defer EXPR
   Defer is used to schedule a function call or block of code to be executed when the surrounding scope exits.
   Deferred actions are executed in reverse order of declaration.

Modules:
    Every file is a module (a compilation unit).
    The module name is derived from its path (so the file name characters are limited to valid identifier characters).
    For make a a simbol visible from other modules use the "pub" keyword before its definition.
    `import std.println` imports the print module from the std module but not re-exports it
    `pub import std.println` re-exports the println function from the print module

Generics:
    Generics are a compile-time feature and are fully monomorphized.
    Generic functions and types generate concrete instances for each used type.
    There is no runtime representation of generics.
    Traits are used as compile-time constraints for generic parameters.
    Traits do not form types and cannot be used as values.
    There is no dynamic dispatch or type erasure.
    All generic constraints are checked at compile time.

Vtables and Dynamic Dispatch:
    The language does not have built-in support for vtables or dynamic dispatch.
    All trait methods are resolved at compile time.
    There is no runtime overhead for trait method calls.
    If dynamic dispatch is needed, it must be implemented manually by the programmer, 
    whether through fat pointers (recommended) or other means.

String Literals and Representation
    Compile-time fixed-size arrays: String literals have a fixed-size array type known at compile time. For example:
    `"hello" has type const [5]u8`
    The type reflects only the characters themselves, not any terminator.

    Null-terminated in memory: The compiler stores string literals in static memory with a null terminator ('\0') appended.
    Memory size for `"hello" = 6 bytes`

    The null byte is not part of the type but is accessible via pointer arithmetic.
    C-string conversion:
    You can get a pointer to the first character as a C-string with zero runtime overhead:
    `const ptr: *const u8 = &"hello";`
  
    This pointer is compatible with C functions, and the null terminator is present in memory.
    No built-in string type: The core language does not include a native string type. Strings are represented either as:
    Fixed-size arrays [N]u8 (primary array type)
    Pointers to null-terminated byte sequences (*const u8)
    Slices and views: Optional slices or views with explicit length can be provided by the standard library when needed.
    Benefits
        Direct C interoperability: Easy to pass string literals to C functions without copies.
        Compile-time length: Known lengths enable safety and optimizations.
        No runtime overhead: No fat pointers or hidden allocations in the core language.
```

// Passing and returning generic values follows normal value semantics.
fn max[T: Ord](a: T, b: T) -> T {
    if a.gt(b) {
        return a
    } else {
        return b
    }
}

fn foo[T]() { } // Error: generic function "foo" is never used

type Int = i32

// This is a tagged union the layout is like a { tag: u8, data: ? } data is the union of all variant data
type Day = enum {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday
}

trait Printable {
    // Self is a compile-time placeholder resolved during trait implementation.
    fn print(self: *Self);
}

trait TwoFunctions {
    fn func1(self: *Self);
    fn func2(self: *Self);
}

type Name = struct {
    first: *u8,
    last: *u8
}

fn print(self: *Name) for Printable {
    println("Name: %s %s", self->first, self->last)
}

fn func1(self: *Name) for TwoFunctions {
    println("Function 1 called for %s %s", self->first, self->last)
}

// error: missing implementation of "func2" for "TwoFunctions" trait on "Name" type

const PI: f32 = 3.14159 // constant at module scope

fn main() i32
{
    let x: i32 = 10 // inmutable binding
    var y: i32 = 20 // mutable binding
    const z: i32 = 30 // compile-time constant

    // Pointer arithmetic
    var f: *i32 = &y
    *f = *f + 5
    let g: *i32 = f + 1 // pointer arithmetic
    var h: *i32 = &y // Error: cannot assign mutable pointer to immutable pointer
    var h: *const i32 = &y // Correct: cast to pointer to const int

    // casting
    var a: f32 = 5.7
    var b: i32 = a as i32 // The cast is explicit except for coercion between const and non-const pointers
    // the conversion between non-const and const pointers is implicit but reverse is not allowed

    // comptime variables
    const size: i32 = 10
    const arr: [size]i32 = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9}

    var person: Name = Name {
        .first = "John",
        .last = "Doe"
    }

    person.print()

    let persona_heap: *Name = malloc<Name>() // malloc return a uninitialized pointer to Name
    defer free(persona_heap)
    
    persona_heap.first = "Jane"
    persona_heap.last = "Smith"
    persona_heap.print()

    let a: i32 = {
        println("Inside block")
    } // Error: block does have a return value so is void

    let value: i32 = {
        defer println("Exiting inner block")
        println("Inside inner block")
        return 42 // The blocks return values ​​with `return`
    } + { return 8 } // You can use blocks or expressions to initialize variables

    let if_expr: i32 = if value > 50 {
        println("Value is greater than 50")
        return 1
    } else {
        println("Value is 50 or less")
        return 0
    }

    var today: Day = Day.Wednesday

    // Match is a exhaustve switch with pattern matching
    match today {
        Day.Sunday, Day.Saturday => {
            println("It's the weekend!")
        },
        _ => {
            println("It's a weekday.")
        }
    }

    return 0
}