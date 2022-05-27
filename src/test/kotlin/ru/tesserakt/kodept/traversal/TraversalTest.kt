package ru.tesserakt.kodept.traversal

import arrow.core.Ior
import arrow.core.Nel
import arrow.core.continuations.eagerEffect
import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeTypeOf
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticWarning
import ru.tesserakt.kodept.error.SyntaxError
import ru.tesserakt.kodept.lexer.CodePoint

class TraversalTest : BehaviorSpec({
    given("effect without shifting") {
        `when`("returns value") {
            then("Ior.Right should be returned") {
                val value = unwrap { eagerEffect { 1 } }
                value.shouldBeTypeOf<Ior.Right<Int>>()
                value.value shouldBe 1
            }
        }

        `when`("writes to sink and returns") {
            then("Ior.Both should be returned") {
                val value = unwrap {
                    eagerEffect {
                        Report(
                            "Hi!",
                            nonEmptyListOf(CodePoint(0, 0)),
                            Report.Severity.WARNING,
                            SemanticWarning.AlignWithType("A")
                        ).report()
                        3
                    }
                }
                value.shouldBeTypeOf<Ior.Both<Nel<Report>, Int>>()
                val (left, right) = value.run { leftValue to rightValue }
                left.head.severity shouldBe Report.Severity.WARNING
                right shouldBe 3
            }
        }
    }

    given("effect with shifts") {
        `when`("shifted with writes") {
            then("Ior.Left will be returned") {
                val value = unwrap {
                    eagerEffect {
                        val i: Int = shift(
                            UnrecoverableError(
                                Report(
                                    "Hi!",
                                    nonEmptyListOf(CodePoint(0, 0)),
                                    Report.Severity.NOTE,
                                    SyntaxError.UnparsedRemainder
                                )
                            )
                        )
                        i
                    }
                }

                value.shouldBeTypeOf<Ior.Left<Nel<Report>>>()
                value.value.head.severity shouldBe Report.Severity.NOTE
            }
        }
    }
})
