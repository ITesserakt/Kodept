package ru.tesserakt.kodept.traversal.inference

import arrow.core.tail
import java.math.BigDecimal
import java.math.BigInteger

sealed interface Language {
    data class Var(val name: String) : Language {
        override fun toString(): String = name
    }

    data class App(val arg: Language, val func: Language) : Language {
        companion object {
            fun curry(args: List<Language>, expr: Language) =
                if (args.isNotEmpty())
                    args.tail().foldRight(App(args.first(), expr), ::App)
                else App(Literal.unit, expr)
        }

        override fun toString(): String = when (arg) {
            is Var, is Literal -> when (func) {
                is Var, is Literal -> "$func $arg"
                is App -> if (func.func is App) "($func) $arg" else "$func $arg"
                else -> "($func) $arg"
            }
            else -> "($func) ($arg)"
        }
    }

    data class Lambda(val bind: Var, val expr: Language) : Language {
        companion object {
            fun uncurry(args: List<Var>, expr: Language) =
                if (args.isNotEmpty())
                    args.dropLast(1).foldRight(Lambda(args.last(), expr), ::Lambda)
                else expr
        }

        override fun toString(): String = "λ$bind. $expr"
    }

    data class Let(val binder: Language, val bind: Var, val usage: Language) : Language {
        override fun toString(): String = "let $bind = $binder in $usage"
    }

    sealed interface Literal : Language {
        data class Number(val value: BigInteger) : Literal {
            override fun toString(): String = value.toString()
        }

        data class Floating(val value: BigDecimal) : Literal {
            override fun toString(): String = value.toString()
        }

        data class Tuple(val items: List<Language>) : Literal {
            override fun toString(): String = items.joinToString(prefix = "(", postfix = ")")
        }

        companion object {
            val unit = Tuple(emptyList())
        }
    }

    override fun toString(): String
}