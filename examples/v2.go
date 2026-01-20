import std.alloc.{allocator, global, arena, global}

type Array[T, A : Allocator] = struct {
	data: *T,
	length: usize,
	capacity: usize,
	allocator: *A,
}

fn new[T, A: Allocator](Array[T, A], allocator: *A) Array[T, A] {
	return Array[T, A] {
		data: null,
		length: 0,
		capacity: 0,
		allocator: allocator,
	}
}

fn push[T, A: Allocator](self: *Array[T, A], value: T) {
	// ...
}

fn main() i32 {
	var arena: Allocator = ArenaAllocator.init(allocator.default)

	var arr1: Array[i32, ArenaAllocator] = Array[i32, ArenaAllocator].new(&arena)
	defer arena.dealloc(arr)

	var arr2: Array[i32, ArenaAllocator] = Array[i32, ArenaAllocator].new(&global)
	defer global.dealloc(arr)

	arr1.push(10)
	arr2.push(20)

	return 0;
}
