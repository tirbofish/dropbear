package com.dropbear.asset

import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

class AssetEntry @OptIn(ExperimentalUuidApi::class) constructor(
    val uuid: Uuid,
    val name: String,
    val assetType: AssetKind,
    val location: ResourceReference,
    val dependencies: List<Uuid>
) {
}