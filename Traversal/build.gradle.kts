plugins {
    kotlin("jvm")
}

group = "ru.tesserakt.kodept"
version = "0.2.1"

val arrowVersion: String by extra
val betterParseVersion: String by extra
val kotestVersion: String by extra

repositories {
    mavenCentral()
}

dependencies {
    implementation(project(":Core"))
    implementation(project(":Misc"))

    testImplementation("io.mockk:mockk:1.12.4")
    testImplementation("io.kotest.extensions:kotest-assertions-arrow:1.2.5")
    testImplementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    testImplementation("io.kotest:kotest-assertions-core:$kotestVersion")
}