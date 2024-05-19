plugins {
    kotlin("jvm")
}

val arrowVersion: String by extra
val betterParseVersion: String by extra
val kotestVersion: String by extra

group = rootProject.group
version = rootProject.version

repositories {
    mavenCentral()
    maven("https://oss.sonatype.org/content/repositories/snapshots/")
}

dependencies {
    api("io.arrow-kt:arrow-core:$arrowVersion")
    api("io.arrow-kt:arrow-eval:$arrowVersion")
    api("io.github.oshai:kotlin-logging:6.0.9")
    implementation("org.jetbrains.kotlinx:kotlinx-collections-immutable:0.3.7")

    testImplementation(kotlin("reflect"))
    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.4.0")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}