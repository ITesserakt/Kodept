package ru.tesserakt.kodept.traversal

import arrow.core.nonEmptyListOf
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeTypeOf
import io.mockk.clearAllMocks
import io.mockk.mockk
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.core.InsecureModifications.withRLT

@OptIn(Internal::class)
class ReferenceResolverTest : StringSpec() {
    private fun Filepath.buildAST(root: AST.Node) = AST(root, this).apply {
        walkThrough {
            when (it) {
                is AST.Reference -> with(mockk<RLT.Reference>(relaxed = true)) { it.withRLT() }
                is AST.InitializedVar -> with(mockk<RLT.InitializedAssignment>(relaxed = true)) { it.withRLT() }
                else -> Unit
            }
        }.forEach { _ -> }
    }

    init {
        beforeTest {
            AST.anyRoot = true
        }

        with(Filepath("TEST FILE")) {
            "reference by initialized variable declaration" {
                val ref = AST.Reference("x")
                val initVar = AST.InitializedVar(ref, false, null, AST.DecimalLiteral(5.toBigInteger()))
                buildAST(initVar)

                unwrap { ReferenceResolver.transform(ref)() }.toEither().shouldBeRight()
                    .toString() shouldBe AST.ResolvedReference(
                    ref.name,
                    initVar,
                    ref.context
                ).toString()
            }

            "reference by variable declaration somewhere in a block" {
                val ref = AST.Reference("x")
                val block = AST.ExpressionList(
                    nonEmptyListOf(
                        AST.InitializedVar(AST.Reference("y"), true, null, AST.StringLiteral("test")),
                        AST.InitializedVar(AST.Reference("x"), false, null, AST.CharLiteral('y')),
                        ref
                    )
                )
                buildAST(block)

                unwrap { ReferenceResolver.transform(ref)() }.toEither()
                    .shouldBeRight().toString() shouldBe AST.ResolvedReference(
                    ref.name,
                    AST.InitializedVar(ref, false, null, AST.CharLiteral('y')),
                    ref.context
                ).toString()
            }

            "reference by function name somewhere in a block" {
                val ref = AST.Reference("x")
                val block = AST.ExpressionList(
                    nonEmptyListOf(
                        AST.ExpressionList(nonEmptyListOf(ref.move())),
                        AST.FunctionDecl("x", emptyList(), null, AST.TupleLiteral.unit)
                    )
                )
                buildAST(block)

                unwrap { ReferenceResolver.transform(ref)() }
                    .toEither().shouldBeRight()
                    .shouldBeTypeOf<AST.ResolvedReference>()
                    .referral.shouldBeTypeOf<AST.FunctionDecl>()
            }
        }

        afterTest { AST.anyRoot = false }
        afterSpec { clearAllMocks() }
    }
}