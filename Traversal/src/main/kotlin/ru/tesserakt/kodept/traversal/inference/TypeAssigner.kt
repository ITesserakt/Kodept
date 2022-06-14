package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.left
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.core.asString
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.KnownType.*
import ru.tesserakt.kodept.traversal.inference.KnownType.Companion.Tag
import ru.tesserakt.kodept.traversal.inference.KnownType.Companion.unit
import ru.tesserakt.kodept.traversal.inference.KnownType.Enum
import ru.tesserakt.kodept.traversal.inference.KnownType.Number

object TypeAssigner : Analyzer() {
    init {
        dependsOn(BinaryOperatorDesugaring, DereferenceTransformer, TypeDereferenceTransformer, UnaryOperatorDesugaring)
    }

    private var uniqueId = 0
        get() = field++

    private val types = hashMapOf<AST.Expression, KnownType>()

    private fun generateUniqueType(): T {
        fun Int.expandToString(alphabet: List<Char> = ('a'..'z').toList()): String {
            if (this == 0) return alphabet[0].toString()
            var current = this
            val sb = StringBuilder()
            while (current > 0) {
                sb.append(alphabet[current % alphabet.size])
                current /= alphabet.size
            }
            return sb.reverse().toString()
        }

        return T("`${uniqueId.expandToString()}")
    }

    private fun AST.TypeReferable.type() = when (this) {
        is AST.EnumDecl.Entry -> Tag(name)
        is AST.EnumDecl -> Enum(name, enumEntries.map { Tag(it.name) })
        is AST.ForeignStructDecl -> Tag(name)
        is AST.StructDecl -> Struct(name, alloc.map { it.type.type() })
        is AST.TraitDecl -> Interface(name)
    }

    private fun AST.TypeLike.type(): KnownType = when (this) {
        is AST.TupleType -> Tuple(items.map { it.type() })
        is AST.ResolvedTypeReference -> referral.type()
        is AST.TypeReference -> TypeDereferenceTransformer.contract()
        is AST.UnionType -> Union(items.map { it.type() })
    }

    private fun AST.Referable.type(): KnownType = when (this) {
        is AST.AbstractFunctionDecl -> Fn(params.map { it.type() }, returns?.type() ?: unit)
        is AST.ForeignFunctionDecl -> Fn(params.map { it.type() }, returns?.type() ?: unit)
        is AST.FunctionDecl -> Fn(params.map { it.type() }, returns?.type() ?: generateUniqueType())
        is AST.InferredParameter -> type?.type() ?: generateUniqueType()
        is AST.InitializedVar -> type?.type() ?: generateUniqueType()
        is AST.Parameter -> type.type()
    }

    private fun smartDeduplicate() {

    }

    context(Filepath) private fun AST.Expression.annotate(): KnownType =
        if (this !in types) {
            when (this) {
                is AST.Dereference -> run {
                    left.annotate()
                    right.annotate()
                    generateUniqueType()
                }

                is AST.BinaryOperator -> BinaryOperatorDesugaring.contract()
                is AST.UnaryOperator -> UnaryOperatorDesugaring.contract()

                is AST.ExpressionList -> {
                    expressions.map {
                        when (it) {
                            is AST.Statement -> it.left()
                            is AST.Expression -> it.annotate()
                        }
                    }
                    generateUniqueType()
                }

                is AST.IfExpr -> {
                    condition.annotate()
                    body.annotate()
                    elifs.map {
                        it.condition.annotate()
                        it.body.annotate()
                        generateUniqueType()
                    }
                    el?.body?.annotate()
                    generateUniqueType()
                }

                is AST.BinaryLiteral -> Number
                is AST.CharLiteral -> KnownType.Char
                is AST.DecimalLiteral -> Number
                is AST.FloatingLiteral -> Floating
                is AST.HexLiteral -> Number
                is AST.OctalLiteral -> Number
                is AST.StringLiteral -> KnownType.String
                is AST.TupleLiteral -> {
                    items.map { it.annotate() }
                    generateUniqueType()
                }

                is AST.FunctionCall -> {
                    reference.annotate()
                    params.map { it.annotate() }
                    generateUniqueType()
                }

                is AST.ResolvedReference -> referral.type()
                is AST.ResolvedTypeReference -> referral.type()
                is AST.TypeReference -> TypeDereferenceTransformer.contract()
                is AST.WhileExpr -> {
                    condition.annotate()
                    body.annotate()
                    generateUniqueType()
                }

                is AST.Reference -> DereferenceTransformer.contract()
            }.also { types += (this to it) }
        } else types[this]!!

    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> {
        println(ast.asString())

        val expressions = ast.fastFlatten(Tree.SearchMode.Preorder).filterIsInstance<AST.Expression>()

        with(ast.filepath) { expressions.forEach { it.annotate() } }
        println(types.map { "${it.key::class.simpleName} -> ${it.value}" }.joinToString("\n"))
        return eagerEffect { }
    }
}