plugins {
    id("kotlin")
}

val kotestVersion: String by extra

group = "ru.tesserakt.kodept"
version = "1.0.0"

repositories {
    mavenCentral()
    maven("https://oss.sonatype.org/content/repositories/snapshots/")
}

dependencies {
    implementation(project(":Core"))
    implementation(kotlin("reflect"))
    implementation("io.kotest:kotest-runner-junit5:$kotestVersion")
    implementation("io.kotest:kotest-assertions-core:$kotestVersion")
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
}