package ru.tesserakt.kodept.core

import io.kotest.assertions.assertSoftly
import io.kotest.core.spec.style.StringSpec
import io.kotest.engine.spec.tempfile
import io.kotest.matchers.sequences.shouldBeLargerThan
import io.kotest.matchers.sequences.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.Tags
import kotlin.io.path.Path

class LoaderTest : StringSpec({
    "load text from scratch" {
        val text = "Hello world!"
        val loader = MemoryLoader.singleSnippet(text)

        loader.getSources() shouldHaveSize 1
        loader.getSources().first().contents.bufferedReader().readText() shouldBe text

        loader.loadSources() shouldHaveSize 1
        loader.loadSources().first().bufferedReader().readText() shouldBe text
    }

    "load from file in linux".config(tags = setOf(Tags.Linux)) {
        val file = tempfile(testCase.descriptor.id.value, ".kd")
        val loader = FileLoader {
            path = Path("/tmp")
        }
        val text = "Hello world!"
        file.writeText(text)

        assertSoftly {
            loader.getSources() shouldHaveSize 1
            loader.loadSources().first().bufferedReader().readText() shouldBe text
        }
    }

    "load any temp file".config(tags = setOf(Tags.Linux)) {
        tempfile(testCase.descriptor.id.value, ".any")
        val loader = FileLoader {
            path = Path("/tmp")
            anySourceExtension = true
        }

        assertSoftly {
            loader.getSources() shouldBeLargerThan sequenceOf(1)
        }
    }
})