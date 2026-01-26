
fn test[A: trait {
	name: str,
	fn print()
}](a: A) {
	a.print()
}

fn test(a: trait {
	name: str,
	fn print()
}) {
	a.print()
}

type Printable = trait {
	name: str,
	fn print()
}

fn test[A: Printable](a: A) {
	a.print()
}
