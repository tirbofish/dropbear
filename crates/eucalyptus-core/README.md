# eucalyptus-core

The core libraries of the eucalyptus editor. Great for embedding into `redback-runtime` and `eucalyptus-editor` as one big change instead of a bunch of features.

It also produces a shared library for Kotlin/Native and the JVM.

Kotlin Multiplatform is prioritised and fully supported on this library, however that is not to say that you (yes you 
the reader) could implement this for another language (as really it just reads from a .dll or a .jar file). 

[//]: # (## Export requirements)

[//]: # ()
[//]: # (To have your own scripting module to be able to be run, you will have to implement the following exports into)

[//]: # (your own app:)

[//]: # ()
[//]: # (- `dropbear_init`)

[//]: # (  - Args: `pointerContext: DropbearContext*`)

[//]: # (  - Returns: `int`)

[//]: # (- `dropbear_load_tagged`)

[//]: # (  - Args: `tag: *mut c_char`)

[//]: # (  - Returns: `int`)

[//]: # (- `dropbear_update_all`)

[//]: # (  - Args: `dt: float`)

[//]: # (  - Returns: `int`)

[//]: # (- `dropbear_update_tagged`)

[//]: # (  - Args: `tag: *mut c_char, `)

[//]: # (  - )