El lenguaje otorgara las funciones basicas sobre las primitivas previamente implementadas, trabajo con tipos numericos y punteros.
Lo demas sera construcciones con el systema de tipos y funciones.

El lenguaje no es memory safe, se espera que el usuario maneje esto a su discrecion, ni contarar con un garbage collector.

Todas las funciones, tipos y/o macros que el lenguaje no provea de forma nativa, seran proporcionadas por la libreria estandar.
No habra soporte nativo para manejo de cadenas de texto, arreglos dinamicos, manejo de memoria, etc, no obstante, la libreria estandar proporcionara estas funcionalidades.
Pero si el usuario desea, puede implementar sus propias versiones de estas funcionalidades y se le dara total control para hacerlo.

## Tipos primitivos (Unicamente numericos)

- Enteros con y sin signo: i8, i16, i32, i64, u8, u16, u32, u64
- Punto flotante: f32, f64
- Booleanos: bool

## Armitmetica de punteros

El lenguaje soportara aritmetica de punteros, permitiendo sumar y restar enteros a punteros.
Esto permitira recorrer arreglos y estructuras de datos en memoria, asi como dereferenciar punteros para acceder a los datos almacenados en memoria o modificar su valor.

```go
let arr: *const i32 = &{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
let ptr: *const i32 = arr + 5  // Apunta al sexto elemento del arreglo
let value: i32 = *ptr          // value es 5
let ptr2: *const i32 = ptr - 2 // Apunta al cuarto elemento del arreglo
let value2: i32 = *ptr2        // value2 es 3
```

No habran arrays o slices como tipos de datos nativos, en su lugar se usaran punteros para trabajar con arreglos y estructuras de datos en memoria.
Es responsabilidad del programador gestionar la memoria y asegurarse de que los punteros y todo lo que se derive de ellos sean validos.

## Literales de Arreglos y Strings

El lenguaje soportara literales de arreglos y cadenas de texto.
Los literales de arreglos y strings permitiran crear arreglos estaticos en memoria, estos literales seran convertidos a arrays con tamaño fijo en tiempo de compilacion.

Las cadenas de texto seran arreglos estaticos de caracteres terminados en null (`\0`) para facilitar su uso con funciones de C y otras librerias que esperen este formato,
pero el usuario podra trabajar con ellas como arreglos normales de caracteres sin necesidad de preocuparse por el null terminator.

```go
let arr: [5]i32 = {1, 2, 3, 4, 5}           // Crea un arreglo estatico de enteros
let ptr: *const i32 = &arr                  // Crea un puntero al arreglo
let first: i32 = *ptr                       // Accede al primer elemento del arreglo
let str: [13]u8 = "Hello, World!"           // Crea una cadena de texto estatica
let str_ptr: *const u8 = &str               // Crea un puntero a la cadena de texto
let char: u8 = *str_ptr                     // Accede al primer caracter de la cadena
```

El tipo de dato de arrays estaticos sera `[N]T`, donde `N` es el tamaño del arreglo y `T` es el tipo de dato de los elementos del arreglo.
Estos tipos solo funcionan como pair de elemento mas tamaño, no habra operaciones nativas para trabajar con ellos, en su lugar si se quiere trabajar con arreglos se debera pasar a punteros y acceder a los elementos mediante aritmetica de punteros.

## Standar library

El lenguaje contara con una libreria estandar basica que proporcionara funciones y tipos para proporcionar lo que el lenguaje no tiene de forma nativa.
Como arreglos estaticos, dinamicos, manejo de cadenas de texto, entrada y salida, manejo de archivos, etc.

```go
import std.Slice

let arr: Slice[5, i32] = Slice.new({1, 2, 3, 4, 5}) // Crea un arreglo estatico de 5 enteros
let value: i32 = arr.at(0)  // Accede al primer elemento del
*arr.at(2) = 10  // Modifica el tercer elemento del arreglo
*arr.get(3) = 20  // Modifica el cuarto elemento del arreglo pero con chequeo de limites
arr.set(4, 30)  // Modifica el quinto elemento del arreglo con chequeo de limites
```

## Conversiones entre punteros const y no const

El lenguaje permitira conversiones entre punteros const y no const.
La conversion entre punteros no const a const sera implicita, pero la conversion inversa no.

## Defer

El lenguaje soportara la palabra clave defer, que permitira posponer la ejecucion de un bloque de codigo hasta que el scope actual termine.
Esto es util para asegurar que ciertos recursos se liberen o ciertas acciones se realicen al final de una funcion, independientemente de como se salga de la funcion.

```go
import "stdio.h"

fn example() {
    let file: *FILE = fopen("data.txt", "r")
    defer close_file(file) // Se ejecutara al final de la funcion

    // Realizar operaciones con el archivo

    // No es necesario llamar a close_file manualmente
}
```

## Generics

El lenguaje soportara genericos, permitiendo definir funciones y tipos que pueden trabajar con diferentes tipos de datos sin necesidad de duplicar codigo.

```go
fn swap[T](a: *T, b: *T) {
    let temp: T = *a
    *a = *b
    *b = temp
}
let x: i32 = 10
let y: i32 = 20
swap(&x, &y) // Ahora x es 20 e y es 10
```

```go
type Pair[T, U] struct {
    first: T,
    second: U,
}
```

Estos sirven tambien para trabajar con traits y constraints en el systema de tipos.

```go
trait Addable {
    fn add(self, other: Self) -> Self
}

fn add[T: Addable](a: T, b: T) -> T {
    return a.add(b)
}
```

## Macros

El lenguaje contara con un sistema de macros que permitira a los usuarios definir macros para generar codigo de forma automatica en tiempo de compilacion.
Las macros seran definidas usando el mismo lenguaje y podran manipular el AST para generar codigo segun las necesidades del usuario.

```go
/// Define a macro
macro log(arg1: AstNode) AstNode {
    // Genera codigo para loguear el valor de arg1
}

// Usa la macro
let x: i32 = 10
#log(x) // Esto expandira a codigo que loguea el valor de x
```

Existe un nodo especial en el AST llamado `AstTokenStream` que representa un flujo de tokens sin parsear y tiene un delimitador especial para indicar el final del stream.
Este nodo es utilizado por las macros para recibir y manipular bloques de codigo sin parsear.
Especialmente util para definir DSLs.

```go
macro dsl(stream: AstTokenStream) AstNode {
    // Aqui se puede parsear el stream manualmente y generar codigo segun las necesidades
    // Por ejemplo para un DSL que convierte html a codigo nativo
    let parser: Parser = Parser.new(stream.tokens)
    let dsl_ast: AstNode = parser.parse_dsl()
    return dsl_ast
}

#dsl `
    <div>
        <h1>Hello, World!</h1>
        <p>This is a DSL example.</p>
    </div>
`
```

## Strict aliasing

El lenguaje implementara strict aliasing, lo que significa que el compilador podra asumir que dos punteros de tipos diferentes no apuntan al mismo objeto en memoria.
Esto permitira optimizaciones mas agresivas por parte del compilador, mejorando el rendimiento del codigo generado.
