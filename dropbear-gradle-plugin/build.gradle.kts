import org.gradle.kotlin.dsl.compileOnly

plugins {
    `kotlin-dsl`
    `maven-publish`
    id("com.gradle.plugin-publish") version "2.0.0"
}

group = "com.dropbear"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("de.undercouch:gradle-download-task:5.6.0")
    compileOnly("org.jetbrains.kotlin:kotlin-gradle-plugin:${KotlinVersion.CURRENT}")
}

gradlePlugin {
    website.set("https://github.com/tirbofish/dropbear")
    vcsUrl.set("https://github.com/tirbofish/dropbear")
    plugins {

        create("dropbearGradlePlugin") {
            id = "dropbear-gradle-plugin"
            implementationClass = "com.dropbear.gradle.DropbearGradlePlugin"
            displayName = "dropbear-gradle-plugin"
            description = "Gradle plugin for dependency management for dropbear-based projects"
            version = version as String
        }
    }
}

publishing {
    publications {
        withType<MavenPublication>().configureEach {
            pom {
                name.set("dropbear-gradle-plugin")
                description.set("Gradle plugin for dropbear dependency management")
                url.set("https://tirbofish.github.io/dropbear/")

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
                developers {
                    developer {
                        id.set("tirbofish")
                        name.set("tk")
                        email.set("tirbofish@pm.me")
                    }
                }
                scm {
                    connection.set("scm:git:git://github.com/tirbofish/dropbear.git")
                    developerConnection.set("scm:git:ssh://github.com/tirbofish/dropbear.git")
                    url.set("https://github.com/tirbofish/dropbear")
                }
            }
        }
    }

    repositories {
        maven {
            name = "GitHubPages"
            url = uri("${layout.buildDirectory}/repo")
        }
    }
}