#![allow(non_snake_case)]

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jlong;
use crate::ptr::CommandBufferPtr;
use crate::command::CommandBuffer;

/**
 * Class:     `com_dropbear_ffi_DropbearEngineNative`
 * 
 * Method:    `quit`
 *
 * Signature: `(J)V`
 *
 * `JNIEXPORT void JNICALL Java_com_dropbear_ffi_DropbearEngineNative_quit
 * (JNIEnv *, jclass, jlong);`
 */
#[unsafe(no_mangle)]
pub fn Java_com_dropbear_ffi_DropbearEngineNative_quit(
    _env: JNIEnv,
    _class: JClass,
    command_handle: jlong,
) {
    let graphics = command_handle as CommandBufferPtr;

    if graphics.is_null() {
        eprintln!(
            "[Java_com_dropbear_ffi_DropbearEngineNative_quit] [ERROR] Graphics pointer is null"
        );
        panic!("NullHandle while quitting GraphicsCommand, better off to shoot with gun than to nicely ask...")
    }

    let graphics = unsafe { &*graphics };

    if let Err(e) = graphics.send(CommandBuffer::Quit) {
        panic!("Unable to send window command while quitting GraphicsCommand, better off to shoot with gun than to nicely ask... \n Error: {}", e);
    }
}