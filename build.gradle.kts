import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
    kotlin("multiplatform") version "1.9.21"
    kotlin("plugin.serialization") version "1.9.21"
}

group = "com.autoxaccount"
version = "0.1.0"

repositories {
    mavenCentral()
}

kotlin {
    // Configure targets for different platforms
    val hostOs = System.getProperty("os.name")
    val isMingwX64 = hostOs.startsWith("Windows")
    val nativeTarget = when {
        hostOs == "Mac OS X" -> {
            val arch = System.getProperty("os.arch")
            if (arch == "aarch64" || arch == "arm64") {
                macosArm64("native")
            } else {
                macosX64("native")
            }
        }
        hostOs == "Linux" -> linuxX64("native")
        isMingwX64 -> mingwX64("native")
        else -> throw GradleException("Host OS is not supported in Kotlin/Native.")
    }

    nativeTarget.apply {
        binaries {
            executable {
                entryPoint = "main"
                baseName = "auto-x-account"
                
                // Static linking configuration - bundle all dependencies
                linkerOpts(
                    "-static-libgcc",
                    "-static-libstdc++",
                )
                
                // Link with pthread for coroutines support
                if (this@apply is KotlinNativeTarget) {
                    when (konanTarget.family) {
                        org.jetbrains.kotlin.konan.target.Family.LINUX,
                        org.jetbrains.kotlin.konan.target.Family.OSX -> {
                            linkerOpts("-lpthread")
                        }
                        else -> {}
                    }
                }
            }
        }
        
        compilations.getByName("main") {
            cinterops {
                // Define C interop configurations if needed for native browser automation
            }
        }
    }

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
                implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.2")
                implementation("org.jetbrains.kotlinx:kotlinx-datetime:0.5.0")
            }
        }
        
        val nativeMain by getting {
            dependencies {
                // Platform-specific dependencies
            }
        }
    }
}

tasks {
    // Clean task
    register("clean") {
        doLast {
            delete(layout.buildDirectory)
        }
    }
}
