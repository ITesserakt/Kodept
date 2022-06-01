plugins {
    kotlin("jvm")
}

val arrowVersion: String by extra
val betterParseVersion: String by extra
val kotestVersion: String by extra

group = "ru.tesserakt.kodept"
version = "0.2.1"

repositories {
    mavenCentral()
}

dependencies {
    api("com.github.h0tk3y.betterParse:better-parse:$betterParseVersion")
    api("io.arrow-kt:arrow-core:$arrowVersion")

    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.2.5")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}