### cleo
Cleo is a transpiler for a extended version of C that add some caracteristics.
It is designed to support c23 features.

#### Features
Over the standard C language, cleo adds:
- [Stronger type system](#type-system)
- [Defer statement](#defer-statement)
- [Declaration order independence](#declaration-order-independence)
- [Associated functions (Methods in types)](#associated-functions)
- [Trait system (comptime interfaces)](#trait-system)
- [Generics (Monomorphization)](#generics)
- [Pattern matching (`match` statement)](#pattern-matching)
- [Modules system](#modules-system)
- And some naming conventions for improved readability and maintainability.

#### Example
```c
// Define a struct with associated functions
typedef struct {
    int x;
    int y;
} Point;

// Associated function to create a new Point
Point new(Point, int x, int y) {
    return (Point){.x = x, .y = y};
}

// Associated function to calculate distance from origin
float distance(Point self) {
    return sqrt(self.x * self.x + self.y * self.y);
}

int main() {
    Point p = Point.new(3, 4);
    float dist = p.distance();
    printf("Distance from origin: %f\n", dist);
    return 0;
}
```

#### Type System
Cleo has a stronger type system than standard C, the conversion between types is more restricted to avoid errors.

The implicit conversions that imply loss or modification of information are forbidden.

```c
int a = 5;
float b = a; // Error: cannot implicitly convert int to float
float c = (float)a; // Correct: explicit conversion
```

Or a change of type itself like from integer to pointer or `int*` to `void*`.

```c
int a = 5;
int* p = &a; // Correct: pointer to int

void* vp = p; // Error: cannot implicitly convert int* to void*
void* vp2 = (void*)p; // Correct: explicit conversion

int* p2 = vp; // Error: cannot implicitly convert void* to int*
int* p3 = (int*)vp; // Correct: explicit conversion
```

Also the coercion between `const` and non-`const` is implicitly when is upgrading but not when is downgrading.

```c
int a = 5;
const int* p1 = &a; // Correct: upgrading to const
int* p2 = (int*)p1; // Error: cannot downgrade from const to non-const
```

#### Defer Statement
Cleo introduces a `defer` statement that allows you to schedule a block of code to be executed when the current scope is exited.

```c
void example() {
    FILE* file = fopen("example.txt", "r");
    defer {
        fclose(file); // This will be executed when the function exits
    };
    // Do something with the file
}
```

#### Declaration Order Independence
In Cleo, you can use functions and types before they are declared. The compiler will handle the order of declarations for you.

```c
int main() {
    int result = add(5, 10); // Using add before its declaration
    printf("Result: %d\n", result);
    return 0;
}

int add(int a, int b) {
    return a + b;
}
```

#### Associated Functions
Cleo allows you to define associated functions (methods) for types, for this is used a special keyword `self` to refer to the instance of the type, if it is used as the first parameter of a function inside a type definition, it will be considered as an associated function.

```c
typedef struct {
    int value;
} Counter;

int increment(Counter self) {
    self.value += 1;
    return self.value;
}

int main() {
    Counter c = {0};
    int newValue = c.increment();
    printf("New value: %d\n", newValue);
    return 0;
}
```

For static associated functions (functions that do not operate on an instance), just use the type name as the first parameter without `self`.

```c
typedef struct {
    int value;
} Counter;

int new(Counter, int initialValue) {
    return (Counter){.value = initialValue};
}

int main() {
    Counter c = Counter.new(10);
    int newValue = c.increment();
    printf("New value: %d\n", newValue);
    return 0;
}
```

#### Trait System (TODO: improve this feature)
Cleo introduces a trait system that allows you to define interfaces that types can implement.
Traits are defined using the `trait` keyword.
To avoid runtime overhead, traits are checked at compile time, ensuring that types conform to the required interface.

```c
trait Printable {
    void print(self);
}

typedef struct {
    int value;
} Number : Printable;

void print(Number self) {
    printf("Number: %d\n", self.value);
}

int main() {
    Number n = {42};
    n.print(); // Calls the print method defined for Number
    return 0;
}
```

If two traits have the same method signature, the type can implement both traits without conflict using the `for` keyword.

```c
trait A {
    void display(self);
}

trait B {
    void display(self);
}

typedef struct {
    int value;
} MyType : A, B;

void display(MyType self) for A {
    printf("A: %d\n", self.value);
}

void display(MyType self) for B {
    printf("B: %d\n", self.value);
}

int main() {
    MyType obj = {10};
    A.display(obj); // Calls A's display
    B.display(obj); // Calls B's display
    return 0;
}
```

#### Generics (TODO)
Cleo supports generics through monomorphization, allowing you to define functions and types that can operate on different data types without runtime overhead.

```c
T max<T>(T a, T b) {
    return (a > b) ? a : b;
}

int main() {
    int intMax = max<int>(5, 10);
    float floatMax = max<float>(5.5, 3.3);
    printf("Max int: %d\n", intMax);
    printf("Max float: %f\n", floatMax);
    return 0;
}
```

TODO: define how to use generics in typesdefs (structs, unions, enums).
```c
typedef struct {
    T value;
} Box<T>;

Box<int> intBox = { 42 };
Box<float> floatBox = { 3.14 };
```

#### Pattern Matching (TODO: muchas cosas que definir aqui)
Cleo introduces a `match` statement that allows you to perform pattern matching on values, similar to switch statements but more powerful.

```c
typedef struct {
    bool is_even;
} EvenOdd;

EvenOdd num = { .is_even = false };
match (num) {
    case EvenOdd { .is_even = true }:
        printf("One\n");
        break;
    case EvenOdd { .is_even = false }:
        printf("Two\n");
        break;
}
```

#### Modules System (TODO)

info a tener en cuenta para desarrollar el sistema de modulos:
En c la palabra `static` se usa para definir el alcance de las funciones y variables a nivel de archivo por lo que seria un `private` en otros lenguajes.

La idea que yo tengo es que los archivos tengan las referencias en si mismo sin tener que agregarlos manualmente, osea que el punto de entrada es el archivo cleo que se quiere compilar y el transpiler se encarga de buscar las dependencias.

Se usara la palabra `include` para importar modulos cleo.
Un modulo cleo sera un archivo cleo o un conjunto de archivos cleo en una carpeta.

```c
include math; // importa el modulo math.cleo o la carpeta math con un archivo module.cleo
#include <stdio.h> // se podria usar para interoperar con archivos c estandar
```

Quedaria definir si usaremos el static para definir funciones y variables privadas al modulo o si usaremos otra palabra reservada como `private` o `export` para definir el alcance como en los modulos de c++.

#### Otras cositas

los enums en c no funcionan como en c, no tiene un namespace propio, por lo que no se puede hacer algo como `MiEnum.A`, habria que pensar en esto como se comportaria dentro cleo.

```c
typedef enum {
    A,
    B,
    C
} MiEnum;

MiEnum value = MiEnum.A; // Error: 'MiEnum' no es un namespace
MiEnum value = A; // Correcto
```