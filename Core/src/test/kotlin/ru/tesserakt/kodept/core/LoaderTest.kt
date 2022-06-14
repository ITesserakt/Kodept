package ru.tesserakt.kodept.core

import Tags
import io.kotest.core.spec.style.StringSpec
import io.kotest.core.spec.style.stringSpec
import io.kotest.engine.spec.tempfile
import io.kotest.matchers.sequences.shouldBeLargerThan
import io.kotest.matchers.sequences.shouldHaveSize
import io.kotest.matchers.shouldBe
import java.io.File
import kotlin.io.path.Path

class LoaderTest : StringSpec({
    assertSoftly = true

    "load text from scratch" {
        val text = "Hello world!"
        val loader = MemoryLoader.singleSnippet(text)

        loader.getSources() shouldHaveSize 1
        loader.getSources().first().contents.bufferedReader().readText() shouldBe text

        loader.loadSources() shouldHaveSize 1
        loader.loadSources().first().bufferedReader().readText() shouldBe text
    }

    fun suite(file: File, filepath: String) = stringSpec {
        val loader = FileLoader {
            path = Path(filepath)
        }
        val text = "Hello world!"
        file.writeText(text)

        loader.getSources() shouldHaveSize 1
        loader.loadSources().first().bufferedReader().readText() shouldBe text
    }

    "load from file in linux by folder".config(tags = setOf(Tags.Linux)) {
        val file = tempfile(testCase.descriptor.id.value, ".kd")
        include(suite(file, "/tmp"))
    }

    "load from file in linux by concrete file".config(tags = setOf(Tags.Linux)) {
        val file = tempfile(testCase.descriptor.id.value, ".kd")
        include(suite(file, "/tmp/${file.name}"))
    }

    "load any temp file".config(tags = setOf(Tags.Linux)) {
        tempfile(testCase.descriptor.id.value, ".any")
        val loader = FileLoader {
            path = Path("/tmp")
            anySourceExtension = true
        }

        loader.getSources() shouldBeLargerThan sequenceOf(1)
    }
})