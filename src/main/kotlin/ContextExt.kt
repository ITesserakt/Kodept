fun <T, U, R> using(a: T, b: U, f: context(T) U.() -> R) = f(a, b)
fun <T, U, V, R> using(a: T, b: U, c: V, f: context(T, U) V.() -> R) = f(a, b, c)