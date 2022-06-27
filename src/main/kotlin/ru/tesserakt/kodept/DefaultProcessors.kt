package ru.tesserakt.kodept

import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.TypeInferenceAnalyzer

val defaultTransformers = setOf(
    TypeSimplifier,
    InitializationTransformer,
    ReferenceResolver,
    VariableScope,
    TypeReferenceResolver,
    ForeignFunctionResolver,
    BinaryOperatorDesugaring,
    UnaryOperatorDesugaring
)

val defaultAnalyzers = setOf(
    moduleNameAnalyzer,
    moduleUniquenessAnalyzer,
    emptyBlockAnalyzer,
    variableUniqueness,
    objectUniqueness,
    TypeInferenceAnalyzer,
)