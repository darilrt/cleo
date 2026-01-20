// file: std/dyn.cleo
struct Dyn[V]{
    data : *void,
    vtable : V,
}

trait IntoDyn[V]
{
    fn into_dyn(self: *Self)->Dyn[V];
}

// file: examples/vtable.cleo
import std.dyn.*;

#dyn
trait Foo : IntoDyn[Foo]
{
    fn foo(self: *Self);
}

// dyn macro generate the vtable and the Foo implementation
// is equal to:
// ```
// struct FooVTable {
//     foo: fn(*void);
// }
//
// fn foo_wrapper[T](data: *void) {
//    Foo.foo(data as *T); // cast back to the original type
// }
// 
// fn from_methods[T](FooVTable, foo: fn(*T)) FooVTable {
//     FooVTable {
//         .foo = foo_wrapper[T],
//     }
// }
//
// fn into_dyn[T](self: FooVTable, t: *T) Dyn[Foo] {
//     Dyn[Foo]{
//         .data = t as *void,
//         .vtable = self,
//     }
// }
// ```

// Now we can use the dynamic dispatch

struct A {
    x: i32,
}

fn foo(self: *A) for Foo {
    println("A.foo: x = {}", self.x);
}

fn into_dyn(self: *A) -> Dyn[Foo] for A {
    FooVTable.from_methods(foo).into_dyn(self)
}