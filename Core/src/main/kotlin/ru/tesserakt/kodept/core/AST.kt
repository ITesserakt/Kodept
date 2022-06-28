@file:Suppress("unused") @file:OptIn(Internal::class)

package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import arrow.core.identity
import arrow.core.prependTo
import kotlinx.collections.immutable.*
import mu.KotlinLogging
import org.jetbrains.annotations.TestOnly
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.Tree.SearchMode
import java.math.BigDecimal
import java.math.BigInteger
import kotlin.properties.Delegates
import kotlin.reflect.KClass
import kotlin.reflect.KProperty

private val logger = KotlinLogging.logger {}

data class AST(private val nodes: PersistentSet<Node>, val filepath: Filepath) {
    val root: Node
    private val parents: ImmutableMap<NodeWithParent, Node> = nodes.flatMap { parent ->
        parent.children().map { it to parent }
    }.associate(::identity).toImmutableMap()

    private fun checkStructure() {
        logger.debug { root.asString() }
        require(parents.size == nodes.size - 1) {
            if (parents.size > nodes.size - 1) "Extra unknown nodes:\n${(parents.keys - nodes).joinToString("\n") { "$it {${it.id}}" }}"
            else "There are missed nodes:\n${(nodes - parents.keys - root).joinToString("\n") { "$it {${it.id}}" }}"
        }

        val graph = OrientedGraph.fromNodes(nodes)
        require(!graph.hasCycles(root)) { "Passed nodes don't form a tree" }
    }

    init {
        root = if (!anyRoot) {
            val roots = nodes.filterIsInstance<NodeWithoutParent>()
            require(roots.size == 1) { "Passed nodes don't form a tree" }
            roots[0]
        } else {
            nodes.first()
        }
        checkStructure()

        nodes.forEach { node ->
            when (node) {
                is NodeBase -> node.ast = this
                is Leaf -> node.ast = this
                else -> Unit
            }
        }
    }

    constructor(root: Node, filepath: Filepath) : this(root.gatherChildren().toPersistentSet(), filepath)

    fun <T> walkThrough(mode: SearchMode = SearchMode.LevelOrder, f: (Node) -> T) = root.walkTopDown(mode, f)

    fun flatten(mode: SearchMode = SearchMode.LevelOrder) = root.gatherChildren(mode)

    inline fun copyWith(modify: Modifications.() -> Unit) = Modifications().apply(modify).build()

    inner class Modifications {
        private val freshNodes: HashSet<Node> = hashSetOf()
        private val changes: HashMap<Node, Node?> = hashMapOf()
        private val adds: HashSet<Node> = hashSetOf()

        private fun Node.addChildren() {
            gatherChildren().forEach { adds += it }
        }

        fun replaced(old: Node, new: Node) {
            changes[old] = new
        }

        fun List<Pair<Node, Node>>.replaced() {
            val grouping = groupBy { it.first }.mapValues { it.value.map(Pair<Node, Node>::second).toMutableList() }
            require(grouping.all { it.value.size == 1 }) { "Multiple transformations found for node" }
            changes += grouping.mapValues { it.value.first() }
        }

        fun Node.deleted() {
            changes[this] = null
        }

        fun Node.added() {
            adds += this
        }

        private fun <T : Node> Iterable<T>.eliminateCells() = map {
            if (it is Cell<*>) it.value else it
        }

        @PublishedApi
        internal fun build(): AST {
            val potentialToUpdate = nodes.filterIsInstance<NodeBase>().flatMap { it.childCells() }

            changes.forEach { (k, v) ->
                if (v !is NodeWithParent) return@forEach
                potentialToUpdate.filter { it.id == k.id }.forEach { cell ->
                    if (cell.id == v.id) cell.value.added()
                    cell.update(v)
                }
            }

            val newNodes = nodes.mutate { mutator ->
                for ((old, new) in changes) {
                    mutator -= old.gatherChildren().toSet()
                    if (new != null) mutator += new.gatherChildren()
                }
                mutator += adds
            }
            return AST(newNodes, filepath)
        }
    }

    companion object {
        val contract = Contract<Node> { "$this is not in AST" }

        @Internal
        @get:TestOnly
        var anyRoot = false

        private var uniqueId = 0
            get() {
                return field++
            }
    }

    sealed interface Node : Tree<Node>, DeepCopyable<Node> {
        override val parent: Node?
        override fun children(): List<NodeWithParent>
        val rlt: RLT.Node
        val id: Int

        override fun equals(other: Any?): Boolean
        override fun hashCode(): Int
        override fun toString(): String
    }

    sealed interface NodeWithParent : Node {
        override val parent: Node

        companion object {
            val contract = Contract<Node> {
                "node $this should always has a parent"
            }
        }
    }

    sealed interface NodeWithoutParent : Node {
        override val parent: Nothing? get() = null
    }

    sealed interface Named : NodeWithParent {
        val name: String
    }

    sealed interface WithResolutionContext : Named, Expression {
        val context: ResolutionContext?

        val fullPath
            get() = "${if (context?.fromRoot == true) "::" else ""}${
                context?.chain?.joinToString("::", postfix = "::") { it.asString() } ?: ""
            }${name}"
    }

    sealed interface FunctionLike : Named {
        val params: List<InferredParameter>
        val returns: TypeLike?
    }

    sealed interface TopLevel : Named
    sealed interface ObjectLevel : Named
    sealed interface StructLevel : ObjectLevel
    sealed interface TraitLevel : ObjectLevel
    sealed interface EnumLevel : ObjectLevel
    sealed interface BlockLevel : NodeWithParent
    sealed interface Expression : BlockLevel
    sealed interface Statement : BlockLevel
    sealed interface Literal : Expression
    sealed interface Lvalue : Expression
    sealed interface Referable : Named
    sealed interface TypeReferable : Named
    sealed interface TypeLike : NodeWithParent
    sealed interface TypeExpression : Node

    class Cell<out T : NodeWithParent>(value: T) : NodeWithParent {
        override val parent get() = value.parent
        override fun children() = value.children()
        override val rlt get() = value.rlt
        override val id get() = value.id
        var value: @UnsafeVariance T = value
            private set

        fun update(new: @UnsafeVariance T) {
            logger.trace("Updated: from $id to ${new.id}")
            this.value = new
        }

        operator fun getValue(self: Any?, prop: KProperty<*>) = value
        override fun equals(other: Any?) = (other as? Node)?.let { it.id == id } ?: false
        override fun hashCode() = id.hashCode()
        override fun toString() = "{$id}"
        override fun deepCopy() = Cell(value.deepCopy() as T)

        companion object {
            private val logger = KotlinLogging.logger { }
        }
    }

    sealed class NodeBase : NodeWithParent {
        final override val parent: Node get() = (ast ?: contract()).parents[this] ?: NodeWithParent.contract()
        internal lateinit var rltSpecial: RLT.Node
        internal var ast: AST? = null
        final override val rlt: RLT.Node get() = rltSpecial

        override val id = uniqueId

        internal abstract fun childCells(): List<Cell<*>>
        final override fun children() = childCells().map { it.value }
        final override fun equals(other: Any?) = (other as? NodeBase)?.let { it.id == id } ?: false
        final override fun hashCode() = id.hashCode()
    }

    sealed class Leaf : NodeWithParent {
        final override val parent get() = (ast ?: contract()).parents[this] ?: NodeWithParent.contract()
        final override fun children() = emptyList<NodeWithParent>()
        internal lateinit var rltSpecial: RLT.Node
        internal var ast: AST? = null
        final override val rlt: RLT.Node get() = rltSpecial

        override val id = uniqueId

        final override fun equals(other: Any?) = (other as? Leaf)?.let { it.id == id } ?: false
        final override fun hashCode() = id.hashCode()
    }

    class Parameter(name: String, override val typeCell: Cell<TypeLike>) : InferredParameter(name, typeCell) {
        override val type by typeCell

        override fun childCells() = listOf(typeCell)
        override fun deepCopy() = with(rlt) { Parameter(name, typeCell.deepCopy()).withRLT() }

        constructor(name: String, type: TypeLike) : this(name, type.move())
    }

    open class InferredParameter(override val name: String, open val typeCell: Cell<TypeLike>?) : NodeBase(),
        Referable {
        open val type get() = typeCell?.value

        override fun childCells() = listOfNotNull(typeCell)
        override fun toString() = "InferredParameter(name='$name', type=$type)"
        override fun deepCopy() = with(rlt) { InferredParameter(name, typeCell?.deepCopy()).withRLT().withRLT() }

        constructor(name: String, type: TypeLike?) : this(name, type?.move())
        constructor(name: String) : this(name, null as? Cell<TypeLike>?)
    }

    data class FileDecl(private val moduleCells: NonEmptyList<Cell<ModuleDecl>>) : NodeWithoutParent {
        val modules by moduleCells
        override var rlt: RLT.File by Delegates.notNull()
            private set

        override val id = uniqueId

        override fun equals(other: Any?) = (other as? FileDecl)?.let { it.id == id } ?: false
        override fun hashCode() = id.hashCode()
        override fun children() = moduleCells.map { it.value }

        context (RLT.File) @Internal
        fun withRLT() = apply { this.rlt = this@File }

        @OptIn(Internal::class)
        override fun deepCopy() = with(rlt) { FileDecl(moduleCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(modules: NonEmptyList<ModuleDecl>) = FileDecl(modules.map { it.move() })
        }
    }

    data class ModuleDecl(override val name: String, val global: Boolean, private val restCells: List<Cell<TopLevel>>) :
        NodeBase(), Named {
        val rest by restCells

        override fun childCells() = restCells
        override fun deepCopy() =
            with(rlt) { ModuleDecl(name, global, restCells.map { it.deepCopy() }).withRLT().withRLT() }

        companion object {
            operator fun invoke(name: String, global: Boolean, rest: List<TopLevel>) =
                ModuleDecl(name, global, rest.move())
        }
    }

    data class StructDecl(
        override val name: String,
        private val allocCells: List<Cell<Parameter>>,
        private val restCells: List<Cell<StructLevel>>,
    ) : NodeBase(), TopLevel, TypeReferable {
        val alloc by allocCells
        val rest by restCells

        override fun childCells() = allocCells + restCells
        override fun deepCopy() =
            with(rlt) { StructDecl(name, allocCells.map { it.deepCopy() }, restCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(name: String, allocs: List<Parameter>, rest: List<StructLevel>) =
                StructDecl(name, allocs.move(), rest.move())
        }
    }

    data class ForeignStructDecl(override val name: String, val relatedWith: String) : Leaf(), TopLevel, TypeReferable {
        override fun deepCopy() = with(rlt) { ForeignStructDecl(name, relatedWith).withRLT() }
    }

    data class EnumDecl(
        override val name: String,
        val stackBased: Boolean,
        private val enumEntryCells: List<Cell<Entry>>,
    ) : NodeBase(), TopLevel, TypeReferable {
        val enumEntries by enumEntryCells

        override fun childCells() = enumEntryCells
        override fun deepCopy() =
            with(rlt) { EnumDecl(name, stackBased, enumEntryCells.map { it.deepCopy() }).withRLT() }

        data class Entry(override val name: String) : Leaf(), EnumLevel, TypeReferable {
            override fun deepCopy() = with(rlt) { Entry(name).withRLT() }
        }

        companion object {
            operator fun invoke(name: String, stackBased: Boolean, enumEntries: List<Entry>) =
                EnumDecl(name, stackBased, enumEntries.move())
        }
    }

    data class TraitDecl(override val name: String, private val restCells: List<Cell<TraitLevel>>) : NodeBase(),
        TopLevel, TypeReferable {
        val rest by restCells

        override fun childCells() = restCells
        override fun deepCopy() = with(rlt) { TraitDecl(name, restCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(name: String, rest: List<TraitLevel>) = TraitDecl(name, rest.move())
        }
    }

    data class AbstractFunctionDecl(
        override val name: String,
        private val paramCells: List<Cell<Parameter>>,
        private val returnCell: Cell<TypeLike>?,
    ) : NodeBase(), TraitLevel, Referable, FunctionLike {
        override val returns get() = returnCell?.value
        override val params by paramCells

        override fun childCells() = paramCells + listOfNotNull(returnCell)
        override fun deepCopy() =
            with(rlt) { AbstractFunctionDecl(name, paramCells.map { it.deepCopy() }, returnCell?.deepCopy()).withRLT() }

        companion object {
            operator fun invoke(name: String, params: List<Parameter>, returns: TypeLike?) =
                AbstractFunctionDecl(name, params.move(), returns?.move())
        }
    }

    data class ForeignFunctionDecl(
        override val name: String,
        private val paramCells: List<Cell<Parameter>>,
        private val returnCell: Cell<TypeReference>?,
        val descriptor: String,
        val action: ExportedFunction = ExportedFunction({ null }, emptyList(), Nothing::class),
    ) : NodeBase(), TopLevel, Referable, FunctionLike {
        override val params by paramCells
        override val returns get() = returnCell?.value

        data class ExportedFunction(
            val action: (List<Any?>) -> Any?,
            val params: List<KClass<*>>,
            val returns: KClass<*>,
        )

        override fun childCells() = paramCells + listOfNotNull(returnCell)
        override fun deepCopy() = with(rlt) {
            ForeignFunctionDecl(
                name, paramCells.map { it.deepCopy() }, returnCell?.deepCopy(), descriptor, action
            ).withRLT()
        }

        val hasAction get() = action.returns != Nothing::class

        companion object {
            operator fun invoke(
                name: String,
                params: List<Parameter>,
                returns: TypeReference?,
                descriptor: String,
                action: ExportedFunction,
            ) = ForeignFunctionDecl(name, params.move(), returns?.move(), descriptor, action)
        }
    }

    data class FunctionDecl(
        override val name: String, private val paramCells: List<Cell<InferredParameter>>,
        private val returnCell: Cell<TypeLike>?, private val restCell: Cell<Expression>,
    ) : NodeBase(), TopLevel, StructLevel, TraitLevel, Referable, FunctionLike, Statement {
        override val params by paramCells
        override val returns get() = returnCell?.value
        val rest by restCell

        override fun childCells() = paramCells + listOfNotNull(returnCell) + listOf(restCell)
        override fun deepCopy() = with(rlt) {
            FunctionDecl(
                name, paramCells.map { it.deepCopy() }, returnCell?.deepCopy(), restCell.deepCopy()
            ).withRLT()
        }

        companion object {
            operator fun invoke(name: String, params: List<InferredParameter>, returns: TypeLike?, rest: Expression) =
                FunctionDecl(name, params.move(), returns?.move(), rest.move())
        }
    }

    data class InitializedVar(
        private val referenceCell: Cell<Reference>,
        val mutable: Boolean,
        private val typeCell: Cell<TypeLike>?,
        private val exprCell: Cell<Expression>,
    ) : NodeBase(), Referable, Statement {
        val reference by referenceCell
        val type get() = typeCell?.value
        val expr by exprCell
        override val name: String = reference.name
        override fun childCells() = listOf(referenceCell) + listOfNotNull(typeCell) + listOf(exprCell)
        override fun deepCopy() = with(rlt) {
            InitializedVar(
                referenceCell.deepCopy(), mutable, typeCell?.deepCopy(), exprCell.deepCopy()
            ).withRLT()
        }

        constructor(reference: Reference, mutable: Boolean, type: TypeLike?, expr: Expression) : this(
            reference.move(),
            mutable,
            type?.move(),
            expr.move()
        )
    }

    data class DecimalLiteral(val value: BigInteger) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { DecimalLiteral(value).withRLT() }
    }

    data class BinaryLiteral(val value: BigInteger) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { BinaryLiteral(value).withRLT() }
    }

    data class OctalLiteral(val value: BigInteger) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { OctalLiteral(value).withRLT() }
    }

    data class HexLiteral(val value: BigInteger) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { HexLiteral(value).withRLT() }
    }

    data class CharLiteral(val value: Char) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { CharLiteral(value).withRLT() }
    }

    data class StringLiteral(val value: String) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { StringLiteral(value).withRLT() }
    }

    data class FloatingLiteral(val value: BigDecimal) : Leaf(), Literal {
        override fun deepCopy() = with(rlt) { FloatingLiteral(value).withRLT() }
    }

    data class TupleLiteral(private val itemCells: List<Cell<Expression>>) : NodeBase(), Literal {
        val items by itemCells

        override fun childCells() = itemCells
        override fun deepCopy() = with(rlt) { TupleLiteral(itemCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(items: List<Expression>) = TupleLiteral(items.move())
            val unit = TupleLiteral(emptyList())
        }
    }

    sealed class BinaryOperator : NodeBase(), Expression {
        protected abstract val leftCell: Cell<Expression>
        protected abstract val rightCell: Cell<Expression>
        val left get() = leftCell.value
        val right get() = rightCell.value
        abstract val kind: OperatorKind

        final override fun childCells() = listOf(leftCell, rightCell)

        sealed interface OperatorKind
    }

    data class Mathematical(
        override val leftCell: Cell<Expression>,
        override val rightCell: Cell<Expression>,
        override val kind: Kind,
    ) : BinaryOperator() {
        enum class Kind : OperatorKind { Add, Sub, Mul, Div, Mod, Pow }

        override fun deepCopy() = with(rlt) { Mathematical(leftCell.deepCopy(), rightCell.deepCopy(), kind).withRLT() }

        constructor(left: Expression, right: Expression, kind: Kind) : this(left.move(), right.move(), kind)
    }

    data class Logical(
        override val leftCell: Cell<Expression>,
        override val rightCell: Cell<Expression>,
        override val kind: Kind,
    ) : BinaryOperator() {
        enum class Kind : OperatorKind { Conjunction, Disjunction }

        override fun deepCopy() = with(rlt) { Logical(leftCell.deepCopy(), rightCell.deepCopy(), kind).withRLT() }

        constructor(left: Expression, right: Expression, kind: Kind) : this(left.move(), right.move(), kind)
    }

    data class Comparison(
        override val leftCell: Cell<Expression>,
        override val rightCell: Cell<Expression>,
        override val kind: Kind,
    ) : BinaryOperator() {
        enum class Kind : OperatorKind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }

        override fun deepCopy() = with(rlt) { Comparison(leftCell.deepCopy(), rightCell.deepCopy(), kind).withRLT() }

        constructor(left: Expression, right: Expression, kind: Kind) : this(left.move(), right.move(), kind)
    }

    data class Binary(
        override val leftCell: Cell<Expression>,
        override val rightCell: Cell<Expression>,
        override val kind: Kind,
    ) : BinaryOperator() {
        enum class Kind : OperatorKind { And, Or, Xor }

        override fun deepCopy() = with(rlt) { Binary(leftCell.deepCopy(), rightCell.deepCopy(), kind).withRLT() }

        constructor(left: Expression, right: Expression, kind: Kind) : this(left.move(), right.move(), kind)
    }

    sealed class UnaryOperator : NodeBase(), Expression {
        protected abstract val exprCell: Cell<Expression>
        val expr get() = exprCell.value

        final override fun childCells() = listOf(exprCell)
    }

    data class Negation(override val exprCell: Cell<Expression>) : UnaryOperator() {
        override fun deepCopy() = with(rlt) { Negation(exprCell.deepCopy()).withRLT() }

        constructor(expr: Expression) : this(expr.move())
    }

    data class Inversion(override val exprCell: Cell<Expression>) : UnaryOperator() {
        override fun deepCopy() = with(rlt) { Inversion(exprCell.deepCopy()).withRLT() }

        constructor(expr: Expression) : this(expr.move())
    }

    data class BitInversion(override val exprCell: Cell<Expression>) : UnaryOperator() {
        override fun deepCopy() = with(rlt) { BitInversion(exprCell.deepCopy()).withRLT() }

        constructor(expr: Expression) : this(expr.move())
    }

    data class Absolution(override val exprCell: Cell<Expression>) : UnaryOperator() {
        override fun deepCopy() = with(rlt) { Absolution(exprCell.deepCopy()).withRLT() }

        constructor(expr: Expression) : this(expr.move())
    }

    data class Assignment(private val leftCell: Cell<Lvalue>, val rightCell: Cell<Expression>) : NodeBase(), Statement {
        val left by leftCell
        val right by rightCell

        override fun childCells() = listOf(leftCell, rightCell)
        override fun deepCopy() = with(rlt) { Assignment(leftCell.deepCopy(), rightCell.deepCopy()).withRLT() }

        constructor(left: Lvalue, right: Expression) : this(left.move(), right.move())
    }

    data class ResolutionContext(val fromRoot: Boolean, val chain: List<Type>)

    open class Reference(override val name: String, override val context: ResolutionContext? = null) : Leaf(), Lvalue,
        Named, WithResolutionContext {
        override fun toString(): String {
            return "Reference(name='$name', resolutionContext=$context)"
        }

        fun copy(name: String = this.name, resolutionContext: ResolutionContext? = this.context) =
            Reference(name, resolutionContext).also { it.rltSpecial = rltSpecial }

        override fun deepCopy() = with(rlt) { Reference(name, context).withRLT() }
    }

    class ResolvedReference(
        name: String,
        private val referralCell: Cell<Referable>,
        resolutionContext: ResolutionContext? = null,
    ) : Reference(name, resolutionContext) {
        val referral by referralCell

        override fun toString() = fullPath
        override fun deepCopy() = with(rlt) { ResolvedReference(name, referralCell.deepCopy(), context).withRLT() }

        constructor(name: String, referral: Referable, context: ResolutionContext? = null) : this(
            name, referral.move(), context
        )
    }

    open class TypeReference(protected val typeCell: Cell<Type>, override val context: ResolutionContext? = null) :
        NodeBase(), Lvalue, TypeLike, WithResolutionContext, Named {
        val type by typeCell
        override val name: String = type.name

        override fun childCells() = listOf(typeCell)
        override fun deepCopy() = with(rlt) { TypeReference(typeCell.deepCopy(), context).withRLT() }
        override fun toString() = fullPath

        constructor(type: Type, context: ResolutionContext? = null) : this(type.move(), context)
        constructor(type: String, context: ResolutionContext? = null) : this(Type(type), context)
    }

    class ResolvedTypeReference(
        type: Cell<Type>,
        private val referralCell: Cell<TypeReferable>,
        context: ResolutionContext? = null,
    ) : TypeReference(type, context) {
        val referral by referralCell

        override fun toString(): String = fullPath
        override fun deepCopy() =
            with(rlt) { ResolvedTypeReference(typeCell.deepCopy(), referralCell.deepCopy(), context).withRLT() }

        constructor(type: Type, referral: TypeReferable, context: ResolutionContext? = null) : this(
            type.move(), referral.move(), context
        )

        constructor(type: String, referral: TypeReferable, context: ResolutionContext? = null) : this(
            Type(type), referral, context
        )
    }

    data class FunctionCall(
        private val referenceCell: Cell<Expression>,
        private val paramCells: List<Cell<Expression>>,
    ) : NodeBase(), Lvalue {
        val reference by referenceCell
        val params by paramCells

        override fun childCells() = referenceCell.prependTo(paramCells)
        override fun deepCopy() =
            with(rlt) { FunctionCall(referenceCell.deepCopy(), paramCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(reference: Expression, params: List<Expression>) =
                FunctionCall(reference.move(), params.move())
        }
    }

    data class Dereference(override var leftCell: Cell<Expression>, override var rightCell: Cell<Expression>) :
        BinaryOperator(), Expression {
        constructor(left: Expression, right: Expression) : this(left.move(), right.move())

        private object DereferenceKind : OperatorKind

        override val kind: OperatorKind = DereferenceKind
        override fun deepCopy() = with(rlt) { Dereference(leftCell.deepCopy(), rightCell.deepCopy()).withRLT() }
    }

    data class ExpressionList(private val expressionCells: NonEmptyList<Cell<BlockLevel>>) : NodeBase(), Expression {
        val expressions by expressionCells

        override fun childCells() = expressionCells
        override fun deepCopy() = with(rlt) { ExpressionList(expressionCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(expressions: NonEmptyList<BlockLevel>) = ExpressionList(expressions.map { it.move() })
        }
    }

    data class Type(override val name: String) : Leaf(), Named, TypeExpression {
        override fun toString() = name
        override fun deepCopy() = with(rlt) { Type(name).withRLT() }
    }

    data class TupleType(private val itemCells: List<Cell<TypeLike>>) : NodeBase(), TypeLike, TypeExpression {
        val items by itemCells

        override fun childCells() = itemCells
        override fun toString() = items.joinToString(prefix = "(", postfix = ")")
        override fun deepCopy() = with(rlt) { TupleType(itemCells.map { it.deepCopy() }).withRLT() }

        companion object {
            val unit = TupleType(mutableListOf())
            operator fun invoke(items: List<TypeLike>) = TupleType(items.move())
        }
    }

    data class UnionType(private val itemCells: NonEmptyList<Cell<TypeLike>>) : NodeBase(), TypeLike, TypeExpression {
        val items by itemCells

        override fun childCells() = itemCells
        override fun toString() = items.joinToString(" | ", "(", ")")
        override fun deepCopy() = with(rlt) { UnionType(itemCells.map { it.deepCopy() }).withRLT() }

        companion object {
            operator fun invoke(items: NonEmptyList<TypeLike>) = UnionType(items.map { it.move() })
        }
    }

    data class IfExpr(
        private val conditionCell: Cell<Expression>,
        private val bodyCell: Cell<Expression>,
        private val elifCells: List<Cell<ElifExpr>>,
        private val elCell: Cell<ElseExpr>?,
    ) : NodeBase(), Expression {
        val condition by conditionCell
        val body by bodyCell
        val elifs by elifCells
        val el get() = elCell?.value

        override fun childCells() = listOf(conditionCell, bodyCell) + elifCells + listOfNotNull(elCell)
        override fun deepCopy() = with(rlt) {
            IfExpr(
                conditionCell.deepCopy(), bodyCell.deepCopy(), elifCells.map { it.deepCopy() }, elCell?.deepCopy()
            ).withRLT()
        }

        companion object {
            operator fun invoke(condition: Expression, body: Expression, elifs: List<ElifExpr>, el: ElseExpr?) =
                IfExpr(condition.move(), body.move(), elifs.move(), el?.move())
        }

        data class ElifExpr(private val conditionCell: Cell<Expression>, private val bodyCell: Cell<Expression>) :
            NodeBase() {
            val condition by conditionCell
            val body by bodyCell

            override fun childCells() = listOf(conditionCell, bodyCell)
            override fun deepCopy() = with(rlt) { ElifExpr(conditionCell.deepCopy(), bodyCell.deepCopy()).withRLT() }

            constructor(condition: Expression, body: Expression) : this(condition.move(), body.move())
        }

        data class ElseExpr(private val bodyCell: Cell<Expression>) : NodeBase() {
            val body by bodyCell

            override fun childCells() = listOf(bodyCell)
            override fun deepCopy() = with(rlt) { ElseExpr(bodyCell.deepCopy()).withRLT() }

            constructor(body: Expression) : this(body.move())
        }
    }

    data class WhileExpr(private val conditionCell: Cell<Expression>, private val bodyCell: Cell<Expression>) :
        NodeBase(), Statement {
        val condition by conditionCell
        val body by bodyCell

        override fun childCells() = listOf(conditionCell, bodyCell)
        override fun deepCopy() = with(rlt) { WhileExpr(conditionCell.deepCopy(), bodyCell.deepCopy()).withRLT() }

        constructor(condition: Expression, body: Expression) : this(condition.move(), body.move())
    }

    data class LambdaExpr(
        private val paramCells: List<Cell<InferredParameter>>,
        private val bodyCell: Cell<Expression>,
        private val returnCell: Cell<TypeLike>?,
    ) : NodeBase(), Expression {
        val params by paramCells
        val body by bodyCell
        val returns get() = returnCell?.value

        override fun childCells() = paramCells + bodyCell + listOfNotNull(returnCell)
        override fun deepCopy() = with(rlt) {
            LambdaExpr(
                paramCells.map { it.deepCopy() }, bodyCell.deepCopy(), returnCell?.deepCopy()
            ).withRLT()
        }

        companion object {
            operator fun invoke(params: List<InferredParameter>, body: Expression, returns: TypeLike?) =
                LambdaExpr(params.move(), body.move(), returns?.move())
        }
    }

    data class ExtensionDecl(
        val typeCell: Cell<TypeReference>,
        val forTraitCell: Cell<TypeReference>,
        val restCell: List<Cell<FunctionDecl>>,
    ) : NodeBase(), TopLevel {
        val type by typeCell
        val forTrait by forTraitCell
        val rest by restCell

        override fun childCells(): List<Cell<*>> = listOf(typeCell, forTraitCell) + restCell
        override fun deepCopy() = with(rlt) {
            ExtensionDecl(typeCell.deepCopy(), forTraitCell.deepCopy(), restCell.map { it.deepCopy() }).withRLT()
        }

        override val name: String get() = "${type.name}\$${forTrait.name}"

        constructor(type: TypeReference, forTrait: TypeReference, rest: List<FunctionDecl>) : this(
            type.move(), forTrait.move(), rest.move()
        )
    }
}

object InsecureModifications {
    context(RLT.Node)
    fun <N : AST.Leaf> N.withRLT() = apply { this.rltSpecial = this@Node }

    context (RLT.Node)
    fun <N : AST.NodeBase> N.withRLT() = apply { this.rltSpecial = this@Node }
}

inline fun <reified T : RLT.Node> AST.Node.accessRLT() = rlt as? T

operator fun <T : AST.NodeWithParent> List<AST.Cell<T>>.getValue(self: Any?, prop: KProperty<*>) = map { it.value }

fun <T : AST.NodeWithParent> T.new() = AST.Cell(this.deepCopy() as T)
fun <T : AST.NodeWithParent> Iterable<T>.new() = map { it.new() }
fun <T : AST.NodeWithParent> T.move() = AST.Cell(this)
fun <T : AST.NodeWithParent> Iterable<T>.move() = map { it.move() }