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
    implementation(project(":Core"))

    implementation("com.github.h0tk3y.betterParse:better-parse:$betterParseVersion")

    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.2.5")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}