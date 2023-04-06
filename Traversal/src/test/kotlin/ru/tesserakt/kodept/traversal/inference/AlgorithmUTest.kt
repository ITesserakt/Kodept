package ru.tesserakt.kodept.traversal.inference

import arrow.core.raise.toEither
import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.traversal.inference.MonomorphicType.*

class AlgorithmUTest : StringSpec({
    infix fun MonomorphicType.testUnify(other: MonomorphicType) =
        (this unify other).toEither()

    "tautology example on constants" {
        val a = Constant(1)
        val b = Constant(1)

        a testUnify b shouldBeRight emptySet()
    }

    "different constants should not unify" {
        val a = Constant(1)
        val b = Constant(2)

        a testUnify b shouldBeLeft Errors.CannotUnify(a, b)
    }

    "tautology example on vars" {
        val a = Var(1)
        val b = Var(1)

        a testUnify b shouldBeRight emptySet()
    }

    "variables should be always unified" {
        val a = Var(1)
        val b = Constant(1)

        a testUnify b shouldBeRight Substitution(a, b).single()
        b testUnify a shouldBeRight Substitution(a, b).single()
    }

    "aliasing" {
        val a = Var(1)
        val b = Var(2)

        a testUnify b shouldBeRight Substitution(a, b).single()
        b testUnify a shouldBeRight Substitution(b, a).single()
    }

    "simple function unifying" {
        val a = Fn.uncurry(Var(1), Constant(1), out = Tuple.unit)
        val b = Fn.uncurry(Var(1), Var(2), out = Tuple.unit)

        a testUnify b shouldBeRight Substitution(Var(2), Constant(1)).single()
    }

    "aliasing in functions" {
        val a = Fn(Var(1), Tuple.unit)
        val b = Fn(Var(2), Tuple.unit)

        a testUnify b shouldBeRight Substitution(Var(1), Var(2)).single()
    }

    "functions with different arity should not unify" {
        val a = Fn(Constant(1), Tuple.unit)
        val b = Fn.uncurry(Constant(1), Constant(2), out = Tuple.unit)

        a testUnify b shouldBeLeft Errors.CannotUnify(Tuple.unit, Fn(Constant(2), Tuple.unit))
    }

    "argument deducing" {
        val a = Fn(Fn(Var(1), PrimitiveType.Number), Tuple.unit)
        val b = Fn(Var(2), Tuple.unit)

        a testUnify b shouldBeRight Substitution(Var(2), Fn(Var(1), PrimitiveType.Number)).single()
    }

    "multiple substitutions" {
        val a = Fn.uncurry(Fn(Var(1), PrimitiveType.Number), Var(1), out = Tuple.unit)
        val b = Fn.uncurry(Var(2), Constant(1), out = Tuple.unit)

        a testUnify b shouldBeRight setOf(
            Substitution(Var(1), Constant(1)),
            Substitution(Var(2), Fn(Constant(1), PrimitiveType.Number))
        )
    }

    "infinite substitution" {
        val a = Var(1)
        val b = Fn(Var(1), Tuple.unit)

        a testUnify b shouldBeLeft Errors.InfiniteType(a, b)
    }

    "transitive substitutions" {
        val a = Var(1)
        val b = Var(2)
        val c = Constant(1)

        val s1 = a testUnify b shouldBeRight Substitution(a, b).single()
        a.substitute(s1) testUnify c shouldBeRight Substitution(b, c).single()

        val s2 = b testUnify c shouldBeRight Substitution(b, c).single()
        a testUnify b.substitute(s2) shouldBeRight Substitution(a, c).single()
    }

    "different substitutions of same variable" {
        val a = Var(1)
        val b = Constant(1)
        val c = Constant(2)

        val s1 = a testUnify b shouldBeRight Substitution(a, b).single()
        a.substitute(s1) testUnify c shouldBeLeft Errors.CannotUnify(b, c)
    }

    "complex unification" {
        val a = Fn(Fn(Fn(Constant(1), Var(2)), Var(3)), Var(4))
        val b = Fn(Var(4), Fn(Var(3), Fn(Var(2), Constant(1))))

        val s1 = (a testUnify b).shouldBeRight()
        val s2 = (b testUnify a).shouldBeRight()

        s1 shouldBe s2
        a.substitute(s1) shouldBe b.substitute(s2)
    }
})
