// The plugin classpath resolves before the project body: activate its lock here.
buildscript {
    configurations.classpath {
        resolutionStrategy.activateDependencyLocking()
    }
}

plugins {
    // AGP 9.0+ has built-in Kotlin support, so the org.jetbrains.kotlin.android
    // plugin is no longer applied (see https://kotl.in/gradle/agp-built-in-kotlin).
    id("com.android.application") version "9.2.1" apply false
}

// Lock transitive application and build-tool graphs. Updating locks is an explicit
// maintenance operation (`--write-locks`), never part of normal developer/CI builds.
allprojects {
    dependencyLocking {
        lockAllConfigurations()
        lockMode = LockMode.STRICT
    }
}
