package ru.tesserakt.kodept.traversal.inference.truely

class TypeContextStep(val context: ContextStep, val type: TypeContext)

class ContextStep(val type: TypeStep, val context: Context) {
    infix fun and(typeContext: TypeContext) = TypeContextStep(this, typeContext)
}

class TypeStep(val assume: AssumeStep, val type: Type) {
    infix fun inContext(context: Context) = ContextStep(this, context)
}

class AssumeStep(val expr: TExpr) {
    infix fun hasType(type: Type) = TypeStep(this, type)
}

class EnsureTypeContextStep(val context: EnsureContextStep, val type: TypeContext)

class EnsureContextStep(val typeStep: EnsureTypeStep, val context: Context) {
    infix fun and(typeContext: TypeContext) = EnsureTypeContextStep(this, typeContext)
}

class EnsureTypeStep(val ensure: EnsureStep, val type: Type) {
    infix fun inContext(context: Context) = EnsureContextStep(this, context)
}

class EnsureStep(val expr: TExpr) {
    infix fun hasType(type: Type) = EnsureTypeStep(this, type)
}

data class Assume(val expr: TExpr, val gamma: Context, val delta: TypeContext, val sigma: Signature)