import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

val betterParseVersion: String by extra
val kotestVersion: String by extra
val arrowVersion: String by extra

plugins {
    kotlin("jvm") version "1.6.20-M1"
    application
}

group = "ru.tesserakt"
version = "0.0.1"

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.github.h0tk3y.betterParse:better-parse:$betterParseVersion")
    implementation("io.arrow-kt:arrow-core:$arrowVersion")

    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
    testImplementation("io.kotest:kotest-property:$kotestVersion")
    testImplementation("io.kotest:kotest-framework-datatest:$kotestVersion")
}

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = "16"
}

tasks.withType<Test> {
    useJUnitPlatform()
}

tasks.withType<Test> {
    systemProperties = System.getProperties().map { it.key.toString() to it.value }.toMap()
}

application {
    mainClass.set("MainKt")
}