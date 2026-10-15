package com.rustcraft.game;

import android.app.Activity;
import android.content.pm.ActivityInfo;
import android.os.Build;
import android.os.Bundle;
import android.util.Log;
import android.view.InputDevice;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

import java.io.File;

/**
 * Main Android Activity hosting the RustCraft native engine.
 * Provides fullscreen immersive layout, surface lifecycle management,
 * high-performance render loop thread, and multi-touch/gamepad event dispatching.
 */
public class RustCraftActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "RustCraftActivity";

    private SurfaceView surfaceView;
    private RenderThread renderThread;
    private boolean isSurfaceReady = false;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Force landscape and keep screen on while playing
        setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);

        // Extend into display cutout / notch area
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            getWindow().getAttributes().layoutInDisplayCutoutMode =
                    WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
        }

        // Apply sticky immersive fullscreen
        hideSystemUI();

        // Initialize Native Engine
        File internalFilesDir = getFilesDir();
        Log.i(TAG, "Initializing RustCraft with internal storage: " + internalFilesDir.getAbsolutePath());
        boolean initOk = RustCraftNative.nativeInit(960, 540, 1337, internalFilesDir.getAbsolutePath());
        if (!initOk) {
            Log.e(TAG, "Failed to initialize RustCraft native engine!");
        }

        surfaceView = new SurfaceView(this);
        surfaceView.getHolder().addCallback(this);
        surfaceView.setFocusable(true);
        surfaceView.setFocusableInTouchMode(true);
        surfaceView.requestFocus();

        setContentView(surfaceView);
    }

    private void hideSystemUI() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            final WindowInsetsController insetsController = getWindow().getInsetsController();
            if (insetsController != null) {
                insetsController.hide(WindowInsets.Type.statusBars() | WindowInsets.Type.navigationBars());
                insetsController.setSystemBarsBehavior(
                        WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                );
            }
        } else {
            getWindow().getDecorView().setSystemUiVisibility(
                    View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                            | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            );
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            hideSystemUI();
        }
    }

    // --- SurfaceHolder.Callback ---

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.i(TAG, "surfaceCreated");
        RustCraftNative.nativeSetSurface(holder.getSurface());
        isSurfaceReady = true;

        if (renderThread == null) {
            renderThread = new RenderThread();
            renderThread.start();
        }
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        Log.i(TAG, "surfaceChanged: " + width + "x" + height);
        RustCraftNative.nativeSetSurface(holder.getSurface());
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.i(TAG, "surfaceDestroyed");
        isSurfaceReady = false;
        RustCraftNative.nativeSetSurface(null);

        if (renderThread != null) {
            renderThread.stopRunning();
            try {
                renderThread.join(500);
            } catch (InterruptedException ignored) {}
            renderThread = null;
        }
    }

    // --- Input Dispatch ---

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        final int action = event.getActionMasked();
        final int pointerIndex = event.getActionIndex();
        final int pointerId = event.getPointerId(pointerIndex);
        final float viewW = surfaceView != null ? (float) surfaceView.getWidth() : 960.0f;
        final float viewH = surfaceView != null ? (float) surfaceView.getHeight() : 540.0f;

        if (action == MotionEvent.ACTION_MOVE) {
            // Dispatch move for all active pointers
            final int pointerCount = event.getPointerCount();
            for (int i = 0; i < pointerCount; i++) {
                int pid = event.getPointerId(i);
                float x = event.getX(i);
                float y = event.getY(i);
                RustCraftNative.nativeTouchEvent(action, pid, x, y, viewW, viewH);
            }
        } else {
            float x = event.getX(pointerIndex);
            float y = event.getY(pointerIndex);
            RustCraftNative.nativeTouchEvent(action, pointerId, x, y, viewW, viewH);
        }

        return true;
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK) {
            // Back button acts as Pause / Menu / Escape
            RustCraftNative.nativeKeyEvent(111, true);
            RustCraftNative.nativeKeyEvent(111, false);
            return true;
        }
        RustCraftNative.nativeKeyEvent(keyCode, true);
        return super.onKeyDown(keyCode, event);
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        RustCraftNative.nativeKeyEvent(keyCode, false);
        return super.onKeyUp(keyCode, event);
    }

    @Override
    public boolean onGenericMotionEvent(MotionEvent event) {
        if ((event.getSource() & InputDevice.SOURCE_JOYSTICK) == InputDevice.SOURCE_JOYSTICK) {
            // Gamepad stick axes can also be forwarded if desired
            return true;
        }
        return super.onGenericMotionEvent(event);
    }

    // --- Lifecycle ---

    @Override
    protected void onPause() {
        super.onPause();
        RustCraftNative.nativeSaveAndPause();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        RustCraftNative.nativeDestroy();
    }

    /**
     * Dedicated render thread targeting 60 FPS frame stepping and blitting.
     */
    private class RenderThread extends Thread {
        private volatile boolean running = true;

        public void stopRunning() {
            running = false;
        }

        @Override
        public void run() {
            long lastTime = System.nanoTime();
            final long targetFrameNs = 1_000_000_000L / 60L; // 60 FPS cap

            while (running) {
                if (isSurfaceReady) {
                    long now = System.nanoTime();
                    float dt = (now - lastTime) / 1_000_000_000.0f;
                    lastTime = now;

                    RustCraftNative.nativeStep(dt);

                    long frameTimeNs = System.nanoTime() - now;
                    long sleepNs = targetFrameNs - frameTimeNs;
                    if (sleepNs > 1_000_000L) {
                        try {
                            Thread.sleep(sleepNs / 1_000_000L);
                        } catch (InterruptedException ignored) {}
                    }
                } else {
                    try {
                        Thread.sleep(16);
                    } catch (InterruptedException ignored) {}
                }
            }
        }
    }
}
