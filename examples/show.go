import c.println

type Printable = trait {
    fn print(self: *Self)
}

type Point[T] = struct {
    x: T,
    y: T
}

fn print[T](self: *Point[T]) {
    println("Point(%d, %d)", self.x, self.y)
}

fn new[T](x: T, y: T) Point[T] {
	return Point[T] {
		.x = x,
		.y = y
	}
}

fn add[T](self: Point[T], b: Point[T]) Point[T] {
    return Point {
        .x = self.x + b.x,
        .y = self.y + b.y
    }
}

fn main() i32 {
	let p1: Point[i32] = Point[i32].new(10, 20)
	var p2: Point[i32] = Point[i32].new(30, 40)
	p2.x = 5

    var p3: Point[i32] = p1.add(p2)
    p3.print()

    return 0
}
