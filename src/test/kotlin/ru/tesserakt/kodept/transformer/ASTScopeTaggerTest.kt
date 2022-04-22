package ru.tesserakt.kodept.transformer

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.core.*

class ASTScopeTaggerTest : BehaviorSpec({
    given("compiler") {
        val compiler = Compiler(MemoryLoader.fromText(sequenceOf(
            """module A =>""",
            """module A {
              |    struct String {
              |         fun concat(self: String, other: String) {
              |             var x = 5
              |             {
              |                 val y = 3
              |                 val z = 2
              |                 assert(x == y + z)
              |             }
              |             x = z
              |         }
              |    }
              |} 
              |module B { } 
              |module C { }""".trimMargin(),
        )))

        `when`("text parsed") {
            val parsed = shouldNotThrowAny { compiler.parse().map { it.toParsedOrThrow().value }.toList() }

            then("getting scope should produce error") {
                shouldThrow<IllegalStateException> { parsed.map { it.root.scope } }
            }

            `when`("ast went through tagger") {
                val newAST = parsed.map { AST(it.root.acceptTransform(ASTScopeTagger()), it.fileName) }

                then("getting scope should not produce error") {
                    newAST.map { it.root.scope shouldBe Scope.Global("") }
                }

                then("scope of elements should properly generated") {
                    val file = newAST.last().root as AST.FileDecl
                    val moduleA = file.modules.head
                    val structString = moduleA.rest.first() as AST.StructDecl
                    val funConcat = structString.rest.first() as AST.FunctionDecl
                    val body = funConcat.rest as AST.ExpressionList
                    val varX = body.expressions[0]
                    val varZ = (body.expressions[1] as AST.ExpressionList).expressions[1]

                    file.scope shouldBe Scope.Global("")
                    moduleA.scope shouldBe Scope.Global("A")
                    structString.scope.parent shouldBe moduleA.scope
                    funConcat.scope.parent shouldBe structString.scope
                    body.scope shouldBe funConcat.scope
                    (varX.scope as Scope.Local).parent shouldBe body.scope
                    (varZ.scope as Scope.Local).parent.parent shouldBe body.scope

                    body.scope isSuperScopeOf varX.scope shouldBe true
                    body.scope isSuperScopeOf varZ.scope shouldBe true
                    varX.scope isSuperScopeOf varZ.scope shouldBe true
                }
            }
        }
    }
})
