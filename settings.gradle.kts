rootProject.name = "Kodept"
include("Core", "Misc", "Traversal")
include("kotest-extensions")

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version("0.4.0")
}
