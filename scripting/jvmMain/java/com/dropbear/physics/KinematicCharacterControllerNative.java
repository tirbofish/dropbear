package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;
import com.dropbear.math.Quaterniond;

// fuck, this got a long ass name
public class KinematicCharacterControllerNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean existsForEntity(long worldHandle, long entityHandle);

    public static native void moveCharacter(long worldHandle, long physicsHandle, long entityHandle, Vector3d translation, double deltaTime);
    public static native void setRotation(long worldHandle, long physicsHandle, long entityHandle, Quaterniond rotation);
    public static native CharacterCollision[] getHit(long worldHandle, long entity);
    public static native CharacterMovementResult getMovementResult(long worldHandle, long entity);
}
