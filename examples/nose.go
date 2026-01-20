
fn test[A: trait {
	fn print()
}](a: A) {
	a.print()
}

type Printable = trait {
	fn print()
}

fn test[A: Printable](a: A) {
	a.print()
}
