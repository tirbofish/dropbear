package com.dropbear.asset

/**
 * A resolved pointer to asset data, handed to the loader.
 *
 * You should never construct this by hand with a raw string —
 * always derive it from an `AssetEntry` via `AssetEntry::to_reference()`, or from a `PackedAssetEntry`
 * via the pak reader.
 *
 * This is the *loading strategy*, not the identity.
 *
 * Identity is always the [kotlin.uuid.Uuid] on [AssetEntry].
*/
sealed class ResourceReference {
    /**
     * A file within the project's `resources/` directory.
     */
    data class File(val path: String) : ResourceReference()

    /**
     * Bytes compiled into the eucalyptus-editor binary via `include_bytes!` (rust macro).
     */
    data class Embedded(val bytes: ByteArray): ResourceReference()

    data class Packed(
        val offset: ULong,
        val length: ULong
    ): ResourceReference()

    /**
     * No backing data; generated entirely at runtime.
     *
     * @param foo Does nothing. It's `null` by default to make the kotlin compiler happy.
     */
    data class Procedural(val foo: Any? = null): ResourceReference()
}