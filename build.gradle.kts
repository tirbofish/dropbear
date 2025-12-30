plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.kotlinxSerialization)
    `maven-publish`
    id("org.jetbrains.dokka") version "2.1.0"
}

group = "com.dropbear"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

val hostOs = providers.systemProperty("os.name").get()
val isArm64 = providers.systemProperty("os.arch").map { it == "aarch64" }.get()
val isMingwX64 = hostOs.startsWith("Windows")
val isLinux = hostOs == "Linux"
val isMacOs = hostOs == "Mac OS X"

val libName = when {
    isMacOs -> "libeucalyptus_core.dylib"
    isLinux -> "libeucalyptus_core.so"
    isMingwX64 -> "eucalyptus_core.dll"
    else -> throw GradleException("Host OS is not supported in Kotlin/Native.")
}

val libPathProvider = provider {
    val candidates = listOf(
        layout.projectDirectory.file("target/debug/$libName").asFile,
        layout.projectDirectory.file("target/release/$libName").asFile,
        layout.projectDirectory.file("libs/$libName").asFile
    )

    candidates.firstOrNull { it.exists() }?.absolutePath ?: ""
}

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

    val nativeLibPathRaw = libPathProvider.get()

    if (nativeLibPathRaw.isNotBlank()) {
        val nativeLibPath = file(nativeLibPathRaw)
        val nativeLibDir = nativeLibPath.parentFile.absolutePath
        val nativeLibFileName = nativeLibPath.name
        val nativeLibNameForLinking = when {
            isMacOs -> nativeLibFileName.removePrefix("lib").removeSuffix(".dylib")
            isLinux -> nativeLibFileName.removePrefix("lib").removeSuffix(".so")
            isMingwX64 -> nativeLibFileName.removeSuffix(".dll")
            else -> throw GradleException("Unsupported OS for library name derivation.")
        }

        nativeTarget.apply {
            compilations.getByName("main") {
                cinterops {
                    val dropbear by creating {
                        defFile(project.file("src/dropbear.def"))
                        includeDirs.headerFilterOnly(project.file("headers"))
                        compilerOpts("-I${project.file("headers").absolutePath}")
                    }
                }
            }
            binaries {
                sharedLib {
                    baseName = "dropbear"
                    if (isLinux || isMacOs) {
                        linkerOpts("-L$nativeLibDir", "-l$nativeLibNameForLinking", "-Wl,-rpath,\\\$ORIGIN")
                    } else if (isMingwX64) {
                        linkerOpts(file("$nativeLibDir/$nativeLibNameForLinking.dll.lib").absolutePath)
                    }
                }
            }
        }
    } else {
        println("WARNING: Rust library not found. Native compilation will skip linking against eucalyptus_core.")
        nativeTarget.apply {
            compilations.getByName("main") {
                cinterops {
                    val dropbear by creating {
                        defFile(project.file("src/dropbear.def"))
                        includeDirs.headerFilterOnly(project.file("headers"))
                    }
                }
            }
        }
    }

    sourceSets {
        commonMain {
            dependencies {
                api("org.jetbrains.kotlinx:kotlinx-datetime:0.7.1")
            }
        }
        nativeMain {
            dependencies {
                implementation(libs.kotlinxSerializationJson)
            }
        }

        jvmMain {
            kotlin.srcDirs("src/jvmMain/kotlin", "build/magna-carta")

            dependencies {
                implementation(kotlin("stdlib"))
            }
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

val extractJavaSources by tasks.registering(Copy::class) {
    group = "jni"
    description = "Copies .java files from jvmMain/kotlin to a temp directory for header generation"

    from("src/jvmMain/kotlin")
    include("**/*.java")
    into(layout.buildDirectory.dir("tmp/java-jni-sources"))
}

val compileJavaForJni by tasks.registering(JavaCompile::class) {
    group = "jni"
    description = "Compiles Java sources and generates JNI headers"

    dependsOn(extractJavaSources, "compileKotlinJvm")

    source(extractJavaSources.map { it.destinationDir })

    classpath = files(
        tasks.named("compileKotlinJvm").map { it.outputs.files },
        configurations.named("jvmCompileClasspath")
    )

    destinationDirectory.set(layout.buildDirectory.dir("tmp/jni-java-classes"))

    val headerOutputDir = layout.buildDirectory.dir("generated/jni-headers")
    options.headerOutputDirectory.set(headerOutputDir)

    doFirst {
        val outDir = headerOutputDir.get().asFile
        if (outDir.exists()) {
            outDir.deleteRecursively()
        }
        outDir.mkdirs()
    }
}

val generateKotlinJniHeaders by tasks.registering(Exec::class) {
    group = "jni"
    description = "Generates JNI headers from Kotlin external functions"

    dependsOn("compileKotlinJvm")

    val headerOutputDir = layout.buildDirectory.dir("generated/jni-headers")
    val kotlinClassesDir = tasks.named("compileKotlinJvm").map {
        it.outputs.files.filter { f -> f.isDirectory }
    }

    inputs.files(kotlinClassesDir)
    outputs.dir(headerOutputDir)

    doFirst {
        val outDir = headerOutputDir.get().asFile
        if (!outDir.exists()) {
            outDir.mkdirs()
        }

        val classFiles = kotlinClassesDir.get().asFileTree
            .filter { it.name.endsWith(".class") && !it.name.contains("$") }
            .files

        if (classFiles.isEmpty()) {
            println("No Kotlin class files found for JNI header generation")
            return@doFirst
        }

        val classpath = kotlinClassesDir.get().joinToString(File.pathSeparator) { it.absolutePath }

        classFiles.forEach { classFile ->
            val baseDir = kotlinClassesDir.get().first { classFile.startsWith(it) }
            val relativePath = classFile.relativeTo(baseDir).path
            val className = relativePath.removeSuffix(".class").replace(File.separatorChar, '.')

            // Use javah (Java 8) or javac -h (Java 9+)
            try {
                exec {
                    commandLine(
                        "javac", "-h", outDir.absolutePath,
                        "-cp", classpath,
                        "-d", layout.buildDirectory.dir("tmp/jni-dummy-classes").get().asFile.absolutePath,
                        classFile.absolutePath
                    )
                    isIgnoreExitValue = true
                }
            } catch (e: Exception) {
                println("Warning: Could not generate header for $className: ${e.message}")
            }
        }

        println("Generated Kotlin JNI Headers at: ${outDir.absolutePath}")
    }
}

val generateJniHeaders by tasks.registering {
    group = "jni"
    description = "Generates all JNI headers (Java and Kotlin)"

    dependsOn(compileJavaForJni, generateKotlinJniHeaders)

    doLast {
        val headerDir = layout.buildDirectory.dir("generated/jni-headers").get().asFile
        val headers = headerDir.listFiles()?.filter { it.extension == "h" } ?: emptyList()
        println("Total JNI headers generated: ${headers.size}")
        headers.forEach { println("  - ${it.name}") }
    }
}

tasks.named("jvmMainClasses") {
    dependsOn(generateJniHeaders)
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
            name.set("dropbear")
            description.set("The dropbear scripting part of the engine")
            url.set("https://github.com/tirbofish/dropbear")

            licenses {
                license {
                    name.set("MIT")
                    url.set("https://opensource.org/license/mit")
                }
                license {
                    name.set("Apache-2.0")
                    url.set("https://www.apache.org/licenses/LICENSE-2.0")
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

tasks.register<Jar>("fatJar") {
    archiveClassifier.set("all")
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE

    from(kotlin.jvm().compilations["main"].output)

    from(configurations.named("jvmRuntimeClasspath").map {
        it.map { file -> if (file.isDirectory) file else zipTree(file) }
    })

    manifest {
        attributes["Implementation-Version"] = project.version
    }
}