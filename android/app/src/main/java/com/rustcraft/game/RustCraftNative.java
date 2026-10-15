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
     * @param width Internal render buffer width (e.g. 960).
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
     * @return 0 on success.
     */
    public static native int nativeStep(float dt);

    /**
     * Dispatches touch down, move, or up events to the multi-touch virtual controls system.
     */
    public static native void nativeTouchEvent(int action, int pointerId, float x, float y, float screenW, float screenH);

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
}
