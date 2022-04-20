package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.core.AST

class DrawProcessor(private val ident: String = "      ") : NodeProcessor<String>() {
    override fun visit(node: AST.WhileExpr): String = """While 
        |   condition:
        |${node.condition.accept(this).prependIndent(ident)}
        |   body:
        |${node.body.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.IfExpr): String = """If
        |   condition:
        |${node.condition.accept(this).prependIndent(ident)}
        |   body:
        |${node.body.accept(this).prependIndent(ident)}
        |${node.elifs.joinToString("\n    elif:", "    elif:") { it.accept(this).prependIndent(ident) }}
        |   else:
        |${node.el?.accept(this).orEmpty().prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.ExpressionList): String = """{
        |   ${node.expressions.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
        |}
    """.trimMargin()

    override fun visit(node: AST.CharLiteral): String = node.value.toString()

    override fun visit(node: AST.BinaryLiteral): String = node.value.toString(2)

    override fun visit(node: AST.DecimalLiteral): String = node.value.toString()

    override fun visit(node: AST.FloatingLiteral): String = node.value.toString()

    override fun visit(node: AST.HexLiteral): String = node.value.toString(16)

    override fun visit(node: AST.OctalLiteral): String = node.value.toString(8)

    override fun visit(node: AST.StringLiteral): String = node.value

    override fun visit(node: AST.Assignment): String = """Assignment
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Binary): String = """Binary(${node.kind})
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Comparison): String = """Comparison(${node.kind})
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Elvis): String = """Elvis
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Logical): String = """Logical(${node.kind})
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Mathematical): String = """Mathematical(${node.kind})
        |   left:
        |${node.left.accept(this).prependIndent(ident)}
        |   right:
        |${node.right.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Absolution): String = """Absolution
        |   expr:
        |${node.expr.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.BitInversion): String = """BitInversion
        |   expr:
        |${node.expr.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Inversion): String = """Inversion
        |   expr:
        |${node.expr.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.Negation): String = """Negation
        |   expr:
        |${node.expr.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.TermChain): String = node.terms.joinToString("\n") { it.accept(this) }

    override fun visit(node: AST.UnresolvedFunctionCall): String = """Function(${node.reference})
        |   params:
        |${node.params.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.UnresolvedReference): String = """Reference(${node.name})"""

    override fun visit(node: AST.TypeExpression): String = """Type(${node.type})"""

    override fun visit(node: AST.FunctionDecl): String = """Function(${node.name})
        |   params:
        |${node.params.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
        |   returns:
        |${node.returns?.accept(this).orEmpty().prependIndent(ident)}
        |   body:
        |${node.rest.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.FunctionDecl.Parameter): String = """Parameter
        |   name:
        |${node.name.prependIndent(ident)}
        |   type:
        |${node.type.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.InitializedVar): String =
        """${if (node.decl.mutable) "Mutable" else "Immutable"} initialized var
        |   variable:
        |${node.decl.accept(this).prependIndent(ident)}
        |   expr:
        |${node.expr.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.VariableDecl): String = """${if (node.mutable) "Mutable" else "Immutable"} var
        |   name:
        |${node.name.prependIndent(ident)}
        |   type:
        |${node.type?.accept(this).orEmpty().prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.FileDecl): String = """File
        |   modules:
        |${node.modules.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.EnumDecl): String = """${if (node.stackBased) "Stack" else "Heap"} enum(${node.name})
        |   entries:
        |${node.enumEntries.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.EnumDecl.Entry): String = """Enum entry(${node.name})"""

    override fun visit(node: AST.ModuleDecl): String = """${if (node.global) "Global m" else "M"}odule(${node.name})
        |   decls:
        |${node.rest.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.StructDecl): String = """Struct(${node.name})
        |   allocated:
        |${node.alloc.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
        |   body:
        |${node.rest.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.StructDecl.Parameter): String = """Parameter
        |   name:
        |${node.name.prependIndent(ident)}
        |   type:
        |${node.type.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.TraitDecl): String = """Trait(${node.name})
        |   body:
        |${node.rest.joinToString("\n") { it.accept(this) }.prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.IfExpr.ElifExpr): String = """Else if
        |   condition:
        |${node.condition.accept(this).prependIndent(ident)}
        |   body:
        |${node.body.accept(this).prependIndent(ident)}
    """.trimMargin()

    override fun visit(node: AST.IfExpr.ElseExpr): String = """Else
        |   body:
        |${node.body.accept(this).prependIndent(ident)}
    """.trimMargin()
}