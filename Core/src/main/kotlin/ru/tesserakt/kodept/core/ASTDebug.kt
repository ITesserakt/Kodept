package ru.tesserakt.kodept.core

fun AST.asString() = root.asString()

fun AST.Node.asString(indent: String = "    "): String = when (this) {
    is AST.Binary -> "${left.asString()} ${kind.name} ${right.asString()}"
    is AST.Comparison -> "${left.asString()} ${kind.name} ${right.asString()}"
    is AST.Dereference -> "${left.asString()}.${right.asString()}"
    is AST.Logical -> "${left.asString()} ${kind.name} ${right.asString()}"
    is AST.Mathematical -> "${left.asString()} ${kind.name} ${right.asString()}"
    is AST.ExpressionList -> """{
        |${expressions.joinToString("\n") { it.asString() }.prependIndent(indent)}
        |}
    """.trimMargin()
    is AST.IfExpr -> """if ${condition.asString()} ${body.asString()}
        |${elifs.joinToString("\n") { it.asString() }}
        |${el?.asString() ?: ""}
    """.trimMargin()
    is AST.BinaryLiteral -> value.toString()
    is AST.CharLiteral -> value.toString()
    is AST.DecimalLiteral -> value.toString()
    is AST.FloatingLiteral -> value.toString()
    is AST.HexLiteral -> value.toString()
    is AST.OctalLiteral -> value.toString()
    is AST.StringLiteral -> value
    is AST.TupleLiteral -> items.joinToString(prefix = "(", postfix = ")") { it.asString() }
    is AST.FunctionCall -> """${reference.asString()}(${this.params.joinToString { it.asString() }})"""
    is AST.Reference -> "${if (context?.fromRoot == true) "::" else ""}${
        context?.chain?.joinToString(
            "::",
            postfix = "::"
        ) { it.asString() } ?: ""
    }${name}"
    is AST.TypeReference -> fullPath
    is AST.Absolution -> "+${expr.asString()}"
    is AST.BitInversion -> "~${expr.asString()}"
    is AST.Inversion -> """!${expr.asString()}"""
    is AST.Negation -> """-${expr.asString()}"""
    is AST.WhileExpr -> """while ${condition.asString()} ${body.asString()}"""
    is AST.Assignment -> """${left.asString()} = ${right.asString()}"""
    is AST.AbstractFunctionDecl -> """abstract fun $name(${params.joinToString { it.asString() }}): ${returns?.asString() ?: AST.TupleType.unit.asString()}"""
    is AST.ForeignFunctionDecl -> """foreign fun $name(${params.joinToString { it.asString() }}): ${returns?.asString() ?: AST.TupleType.unit.asString()} => $descriptor"""
    is AST.FunctionDecl -> """fun $name(${params.joinToString { it.asString() }})${if (returns != null) ": ${returns!!.asString()}" else ""} ${rest.asString()}"""
    is AST.Parameter -> """$name: ${type.asString()}"""
    is AST.InferredParameter -> """$name${if (type != null) ": ${type!!.asString()}" else ""}"""
    is AST.InitializedVar -> """${if (mutable) "var" else "val"} $name${if (type != null) ": ${type!!.asString()}" else ""} = ${expr.asString()}"""
    is AST.EnumDecl.Entry -> name
    is AST.ForeignStructDecl -> """foreign type $name => "$relatedWith""""
    is AST.ModuleDecl -> """module $name ${if (global) "=>" else "{"}
        |${rest.joinToString("\n") { it.asString().prependIndent(indent) }}
        |${if (global) "}" else ""}${if (global) "" else "\n}"}
    """.trimMargin()
    is AST.EnumDecl -> """enum ${if (stackBased) "struct" else "class"} $name {
        |${enumEntries.joinToString { it.asString() }.prependIndent(indent)}
        |}
    """.trimMargin()
    is AST.StructDecl -> """struct $name(${alloc.joinToString { it.asString() }}) {
        |${rest.joinToString("\n") { it.asString() }.prependIndent(indent)}
        |}
    """.trimMargin()
    is AST.TraitDecl -> """trait $name {
        |${rest.joinToString("\n") { it.asString() }.prependIndent(indent)}
        |}
    """.trimMargin()
    is AST.Type -> toString()
    is AST.IfExpr.ElifExpr -> "elif ${condition.asString()} ${body.asString()}"
    is AST.IfExpr.ElseExpr -> "else ${body.asString()}"
    is AST.FileDecl -> modules.joinToString("\n") { it.asString() }
    is AST.TupleType -> toString()
    is AST.UnionType -> toString()
    is AST.LambdaExpr -> """\(${params.joinToString { it.asString() }}) -> ${body.asString()}"""
    is AST.Cell<*> -> value.asString()
}