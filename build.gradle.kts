import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

val betterParseVersion: String by extra
val kotestVersion: String by extra
val arrowVersion: String by extra
val serializationVersion: String by extra

plugins {
    kotlin("jvm") version "1.7.0-RC2"
    application
}

group = "ru.tesserakt"
version = "0.2.1"

repositories {
    mavenCentral()
}

dependencies {
    implementation(project(":Core"))
    implementation(project(":Misc"))
    implementation(project(":Traversal"))

    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.2.5")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}

allprojects {
    tasks.withType<KotlinCompile> {
        kotlinOptions.jvmTarget = "15"
        kotlinOptions.freeCompilerArgs += "-Xcontext-receivers"
    }

    tasks.withType<Test> {
        useJUnitPlatform()
        systemProperties = System.getProperties().map { it.key.toString() to it.value }.toMap()
    }
}

application {
    mainClass.set("MainKt")
}