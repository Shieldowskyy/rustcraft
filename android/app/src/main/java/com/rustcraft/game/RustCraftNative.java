package com.rustcraft.game;

import android.view.Surface;

/**
 * JNI bindings for the RustCraft native game engine.
 */
public class RustCraftNative {
    static {
        System.loadLibrary("rustcraft");
    }

    /**
     * Initializes the RustCraft game engine state machine.
     *
     * @param width Internal render buffer width (e.g. 960 or dynamic aspect ratio width).
     * @param height Internal render buffer height (e.g. 540).
     * @param seed World generation seed.
     * @param internalFilesDir Absolute path to the app's internal files directory for saves.
     * @return true if initialization succeeded.
     */
    public static native boolean nativeInit(int width, int height, int seed, String internalFilesDir);

    /**
     * Binds or unbinds an Android Surface to ANativeWindow.
     * Pass null when surface is destroyed.
     */
    public static native void nativeSetSurface(Surface surface);

    /**
     * Advances game simulation and renders a frame directly to ANativeWindow.
     *
     * @param dt Delta time in seconds (or 0.0 to auto-measure).
     * @return Status bitmask: Bit 0 (1) = Chat active / text input requested, Bit 1 (2) = Menu active.
     */
    public static native int nativeStep(float dt);

    /**
     * Dispatches touch down, move, or up events to the multi-touch virtual controls system.
     */
    public static native void nativeTouchEvent(int action, int pointerId, float x, float y, float screenW, float screenH);

    /**
     * Dispatches physical gamepad analog sticks, triggers, and D-pad axes.
     */
    public static native void nativeAxisEvent(float axisX, float axisY, float axisZ, float axisRz, float axisLt, float axisRt, float axisHatX, float axisHatY);

    /**
     * Dispatches typed text from Android software keyboard / IME into the active chat prompt.
     */
    public static native void nativeTextInput(String text);

    /**
     * Checks if text input (chat/command prompt) is currently waiting for keyboard input.
     */
    public static native boolean nativeIsTextInputActive();

    /**
     * Dispatches hardware keyboard or gamepad button events.
     */
    public static native void nativeKeyEvent(int keyCode, boolean isDown);

    /**
     * Autosaves world data when activity is paused or sent to background.
     */
    public static native void nativeSaveAndPause();

    /**
     * Cleans up native engine resources and saves world.
     */
    public static native void nativeDestroy();

    /**
     * Returns the active renderer backend name ("opengl", "vulkan", "android_software", etc.).
     */
    public static native String nativeGetRendererBackend();

    /**
     * Checks if the active renderer is hardware-accelerated (OpenGL / Vulkan).
     * Returns false if the game fell back to software rasterization.
     */
    public static native boolean nativeIsHardwareAccelerated();
}
