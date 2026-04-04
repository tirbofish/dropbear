plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.kotlinxSerialization)
    `maven-publish`
    id("com.gradleup.shadow") version "9.2.2"
}

group = "io.github.tirbofish"
version = "0.1.0"

repositories {
    mavenCentral()
    mavenLocal() // kept for local development, change to the maven URI.
//    maven {url = uri("https://tirbofish.github.io/dropbear/") }
}

val hostOs = providers.systemProperty("os.name").get()
val isArm64 = providers.systemProperty("os.arch").map { it == "aarch64" }.get()
val isMingwX64 = hostOs.startsWith("Windows")
val isLinux = hostOs == "Linux"
val isMacOs = hostOs == "Mac OS X"

kotlin {
    jvm { }

    val nativeTarget = when {
        isMacOs && isArm64 -> macosArm64("nativeLib")
        isMacOs && !isArm64 -> macosX64("nativeLib")
        isLinux && isArm64 -> linuxArm64("nativeLib")
        isLinux && !isArm64 -> linuxX64("nativeLib")
        isMingwX64 -> mingwX64("nativeLib")
        else -> throw GradleException("Host OS is not supported in Kotlin/Native.")
    }

    // --- plugin library resolution (mirrors root build.gradle.kts pattern) ---
    val pluginLibName = when {
        isMacOs    -> "libsurface_nets_plugin.dylib"
        isLinux    -> "libsurface_nets_plugin.so"
        isMingwX64 -> "surface_nets_plugin.dll"
        else       -> throw GradleException("Unsupported OS for library name derivation.")
    }

    val pluginLibPathProvider = provider {
        val candidates = listOf(
            layout.projectDirectory.file("../../target/debug/$pluginLibName").asFile,
            layout.projectDirectory.file("../../target/release/$pluginLibName").asFile,
        )
        candidates.firstOrNull { it.exists() }?.absolutePath ?: ""
    }

    val pluginLibPathRaw = pluginLibPathProvider.get()

    if (pluginLibPathRaw.isNotBlank()) {
        val pluginLibPath = file(pluginLibPathRaw)
        val pluginLibDir  = pluginLibPath.parentFile.absolutePath
        val pluginLibLinkName = when {
            isMacOs    -> pluginLibPath.name.removePrefix("lib").removeSuffix(".dylib")
            isLinux    -> pluginLibPath.name.removePrefix("lib").removeSuffix(".so")
            isMingwX64 -> pluginLibPath.name.removeSuffix(".dll")
            else       -> throw GradleException("Unsupported OS for link name derivation.")
        }

        nativeTarget.apply {
            compilations.getByName("main") {
                cinterops {
                    val surfaceNetsPlugin by creating {
                        defFile(project.file("scripting/surface_nets_plugin.def"))
                        includeDirs.headerFilterOnly(project.file("include"))
                        compilerOpts("-I${project.file("include").absolutePath}")
                    }
                }
            }
            binaries.all {
                if (isLinux || isMacOs) {
                    linkerOpts("-L$pluginLibDir", "-l$pluginLibLinkName", "-Wl,-rpath,\\\$ORIGIN")
                } else if (isMingwX64) {
                    linkerOpts("$pluginLibDir/${pluginLibLinkName}.dll.lib")
                }
            }
        }
    } else {
        println("Skipping surface-nets-plugin native cinterop: plugin library not found in target/.")
    }

    sourceSets {
        commonMain {
            kotlin.srcDirs("scripting/commonMain")
            dependencies {
                api("org.jetbrains.kotlinx:kotlinx-datetime:0.7.0")
                api("com.dropbear:dropbear:1.0-SNAPSHOT")
            }
        }

        jvmMain {
            kotlin.srcDirs("scripting/jvmMain/kotlin")
            dependencies {
                api("com.dropbear:dropbear:1.0-SNAPSHOT")
            }
        }

        nativeMain {
            kotlin.srcDirs("scripting/nativeMain/kotlin")
            dependencies {
                api("com.dropbear:dropbear:1.0-SNAPSHOT")
            }
        }
    }

    java {
        sourceSets.getByName("jvmMain") {
            java.srcDirs("scripting/jvmMain/java")
        }
    }

    targets.all {
        compilations.all {
            compileTaskProvider.configure {
                compilerOptions {
                    freeCompilerArgs.add("-Xexpect-actual-classes")
                }
            }
        }
    }
}

tasks.register<JavaCompile>("generateJniHeaders") {
    val outputDir = layout.buildDirectory.dir("generated/jni-include")
    options.headerOutputDirectory.set(outputDir.get().asFile)

    destinationDirectory.set(layout.buildDirectory.dir("classes/java/jni"))

    classpath = files(
        tasks.named("compileKotlinJvm"),
    )

    source = fileTree("scripting/jvmMain/java") {
        include("**/*.java")
    }

    dependsOn("compileKotlinJvm")

    doFirst {
        val javaFiles = source.files
        if (javaFiles.isEmpty()) {
            println("WARNING: No Java files found in scripting/jvmMain/java for JNI header generation")
        } else {
            println("Generating JNI headers for \${javaFiles.size} Java files:")
            javaFiles.forEach { println("  - \${it.name}") }
        }
    }

    doLast {
        val headerDir = outputDir.get().asFile
        val headers = headerDir.listFiles()?.filter { it.extension == "h" } ?: emptyList()
        println("Generated \${headers.size} JNI headers:")
        headers.forEach { println("  - \${it.name}") }
    }
}

publishing {
    repositories {
        maven {
            name = "GitHubPages"
            url = uri(layout.buildDirectory.dir("repo"))
        }
    }

    publications.withType<MavenPublication> {
        pom {
            name.set("surface-nets-plugin")
            description.set("Surface Nets isosurface plugin for the dropbear engine.")
            url.set("https://github.com/tirbofish/dropbear")

            licenses {
                license {
                    name.set("MIT License")
                    url.set("https://mit-license.org/")
                }
            }
            
            developers {
                developer {
                    id.set("tirbofish")
                    name.set("tk")
                    email.set("4tkbytes@pm.me")
                }
            }

            scm {
                url.set("https://github.com/tirbofish/dropbear")
                connection.set("scm:git:git://github.com/tirbofish/dropbear.git")
                developerConnection.set("scm:git:ssh://git@github.com/tirbofish/dropbear.git")
            }
        }
    }
}
