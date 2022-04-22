package ru.tesserakt.kodept.core

import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.shouldBe
import java.util.*

class ScopeTest : BehaviorSpec({
    given("scope graph") {
        val root1 = Scope.Global("A")
        val ancestor1 = Scope.Object(root1, UUID(0, 1))
        val ancestor2 = Scope.Local(ancestor1, UUID(1, 0))
        val ancestor3 = Scope.Object(root1, UUID(1, 1))
        val ancestor4 = Scope.Local(ancestor1, UUID(10, 0))
        val leaf5 = Scope.Local(ancestor4, UUID(10, 1))
        val ancestor6 = Scope.Local(ancestor3, UUID(11, 0))
        val leaf7 = Scope.Local(ancestor3, UUID(11, 1))
        val leaf8 = Scope.Local(ancestor6, UUID(100, 0))
        val ancestor9 = Scope.Local(ancestor2, UUID(100, 1))
        val leaf10 = Scope.Local(ancestor9, UUID(101, 0))
        val leaf11 = Scope.Local(ancestor9, UUID(101, 1))
        val leaf12 = Scope.Object(root1, UUID(110, 0))
        val leaf13 = Scope.Local(ancestor1, UUID(110, 1))

        val root2 = Scope.Global("B")
        val ancestor14 = Scope.Object(root2, UUID(111, 0))
        val leaf15 = Scope.Local(ancestor14, UUID(111, 1))

        val scopeList = listOf(ancestor1,
            ancestor2,
            ancestor3,
            ancestor4,
            leaf5,
            ancestor6,
            leaf7,
            leaf8,
            ancestor9,
            leaf10,
            leaf11,
            leaf12,
            leaf13)

        `when`("checking sub nodes") {
            then("nodes 9, 10, 11 should be sub nodes of node 2") {
                ancestor9 isSubScopeOf ancestor2 shouldBe true
                leaf10 isSubScopeOf ancestor2 shouldBe true
                leaf11 isSubScopeOf ancestor2 shouldBe true
            }

            then("root1 node should not be sub node of node 2") {
                root1 isSubScopeOf ancestor2 shouldBe false
            }

            then("nodes 6 and 13 should not be sub node of node 2") {
                ancestor6 isSubScopeOf ancestor2 shouldBe false
                leaf13 isSubScopeOf ancestor2 shouldBe false
            }

            then("any node except 14 and 15 should be sub node of root1") {
                scopeList.forAll { it isSubScopeOf root1 && !(it isSubScopeOf root2) }
                ancestor14 isSubScopeOf root1 shouldBe false
                leaf15 isSubScopeOf root1 shouldBe false
            }

            then("node 15 should not be sub node of node 1") {
                leaf15 isSubScopeOf ancestor1 shouldBe false
            }

            then("node is sub node of itself") {
                scopeList.forAll { it isSubScopeOf it shouldBe true }
            }
        }

        `when`("checking super nodes") {
            then("node 2 should be super node of nodes 9, 10, 11") {
                ancestor2 isSuperScopeOf ancestor9 shouldBe true
                ancestor2 isSuperScopeOf leaf10 shouldBe true
                ancestor2 isSuperScopeOf leaf11 shouldBe true
            }

            then("node2 should not be super node of root1 node") {
                ancestor2 isSuperScopeOf root1 shouldBe false
            }

            then("node 2 should not be super node of nodes 6 and 13") {
                ancestor2 isSuperScopeOf ancestor6 shouldBe false
                ancestor2 isSuperScopeOf leaf13 shouldBe false
            }

            then("root1 should be super node for any node except 14 and 15") {
                scopeList.forAll { root1 isSuperScopeOf it && !(root2 isSuperScopeOf it) }
                root1 isSuperScopeOf ancestor14 shouldBe false
                root1 isSuperScopeOf leaf15 shouldBe false
            }

            then("node 1 should not be super node of node 15") {
                ancestor1 isSuperScopeOf leaf15 shouldBe false
            }

            then("node is super node of itself") {
                scopeList.forAll { it isSuperScopeOf it shouldBe true }
            }
        }

        `when`("checking common ancestor") {
            then("c.a. of itself should be itself") {
                scopeList.forAll { it commonAncestor it shouldBe it }
            }

            then("c.a. of nodes 6 and 7 is node 3") {
                ancestor6 commonAncestor leaf7 shouldBe ancestor3
            }

            then("c.a of roots should be null") {
                root1 commonAncestor root2 shouldBe null
            }

            then("c.a of node 5 and node 11 should be node 1") {
                leaf11 commonAncestor leaf5 shouldBe ancestor1
            }

            then("c.a. of node 10 and node 8 should be root1") {
                leaf10 commonAncestor leaf8 shouldBe root1
            }

            then("c.a. of node 7 and node 14 should be null") {
                leaf7 commonAncestor ancestor14 shouldBe null
            }
        }
    }
})
