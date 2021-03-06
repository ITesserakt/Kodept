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
}

dependencies {
    implementation(project(":Core"))
    implementation(project(":Misc"))

    testImplementation(project(":kotest-extensions"))
    testImplementation("io.mockk:mockk:1.12.4")
    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.2.5")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}