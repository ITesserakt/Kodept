package ru.tesserakt.kodept.traversal.inference.truely

sealed class TExpr {
    data class TVal<V : TValue>(val value: V) : TExpr()
    data class TApp(val a: TExpr, val b: TExpr) : TExpr()
    data class TTypeApp(val a: TExpr, val b: Type) : TExpr()
    data class TVar(val id: Int) : TExpr()

    fun toTypeLambda() = TVal(TValue.TTypeLambda(this))
    fun toLambda(type: Type) = TVal(TValue.TLambda(type, this))
}

sealed interface TValue {
    data class TLambda(val type: Type, val expr: TExpr) : TValue
    data class TTypeLambda(val expr: TExpr) : TValue
    data class TConst(val id: Int) : TValue
}