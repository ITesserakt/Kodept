import io.kotest.core.config.AbstractProjectConfig
import io.kotest.core.names.DuplicateTestNameMode
import io.kotest.core.names.TestNameCase
import io.kotest.core.spec.SpecExecutionOrder

class TestConfig : AbstractProjectConfig() {
    override val parallelism = 4
    override val testNameAppendTags = true
    override val testNameCase = TestNameCase.Sentence
    override val specExecutionOrder = SpecExecutionOrder.FailureFirst
    override val duplicateTestNameMode = DuplicateTestNameMode.Silent
}