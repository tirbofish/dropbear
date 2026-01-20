package com.dropbear

import com.dropbear.utils.ID

/**
 * The ID of an entity (represented as a [Long])
 *
 * @property raw The entity into bits.
 */
data class EntityId(val raw: Long): ID(raw)