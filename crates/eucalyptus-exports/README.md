# eucalyptus-exports

A shared library that absracts the exports from `eucalyptus-core` and presents them in as `libeucalyptus_core.so` 
(or the equivalent for the related platform). 

## Language Support

Despite the project using ECS, OOP-based paradigms are used heavily (in Kotlin) and can be used for your own bindings. 

Currently, this engine has support in these languages (priority in order)

### JVM
Primarily JVM-based languages (Kotlin/JVM first-class, Java available).
#### Caveats
- Some issues with JVM is that the final executable that is shipped will need to be enabled 
through an executable argument `{} --enable-jvm` (or enabled through a shipping setting within Project Settings). 
Within the dropbear project, the JVM is used primarily for editor plugins and easy game prototyping. 
C-Native-based languages can be used as an alternative for prototyping, however, it will require a longer build time 
and will require the binding developer to create their own reflection methods. 

### C-Native
It is primarily supported through Kotlin/Native; however, the same exports *can* be used with
other languages, such as Python (no one has made such a thing). Follow the [Kotlin](../../scripting) class
structure for OOP based languages (idk about other paradigm based languages). 

There are headings created (albeit not always updated) under [dropbear.h](../../include/dropbear.h) that you can
use for reference or your own bindings. 