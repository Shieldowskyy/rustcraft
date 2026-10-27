package com.rustcraft.game;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.os.Build;
import android.os.Bundle;
import android.text.InputType;
import android.widget.Toast;
import android.util.DisplayMetrics;
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
import android.view.inputmethod.BaseInputConnection;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;

import java.io.File;

/**
 * Main Android Activity hosting the RustCraft native engine.
 * Provides fullscreen immersive layout, surface lifecycle management,
 * high-performance render loop thread, dynamic aspect-ratio scaling,
 * software keyboard (IME) integration, and multi-touch/gamepad event dispatching.
 */
public class RustCraftActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "RustCraftActivity";

    private GameSurfaceView surfaceView;
    private RenderThread renderThread;
    private boolean isSurfaceReady = false;
    private volatile boolean isKeyboardVisible = false;

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

        surfaceView = new GameSurfaceView(this);
        surfaceView.getHolder().addCallback(this);
        surfaceView.setFocusable(true);
        surfaceView.setFocusableInTouchMode(true);
        surfaceView.requestFocus();

        setContentView(surfaceView);

        // Calculate optimal render resolution matching device aspect ratio
        DisplayMetrics dm = getResources().getDisplayMetrics();
        int screenW = Math.max(dm.widthPixels, dm.heightPixels);
        int screenH = Math.min(dm.widthPixels, dm.heightPixels);

        // Standard render height is 540p (crisp retro aesthetic, locked 60 FPS)
        int renderH = 540;
        int renderW = Math.max(960, Math.round(540.0f * ((float) screenW / (float) screenH)));
        // Align to 4-pixel multiple for optimal memory alignment
        renderW = (renderW + 3) & ~3;

        // Initialize Native Engine
        File internalFilesDir = getFilesDir();
        Log.i(TAG, "Initializing RustCraft (" + renderW + "x" + renderH + ") with saves at: " + internalFilesDir.getAbsolutePath());
        boolean initOk = RustCraftNative.nativeInit(renderW, renderH, 1337, internalFilesDir.getAbsolutePath());
        if (!initOk) {
            Log.e(TAG, "Failed to initialize RustCraft native engine!");
        } else if (!RustCraftNative.nativeIsHardwareAccelerated()) {
            String backend = RustCraftNative.nativeGetRendererBackend();
            Log.w(TAG, "RustCraft is running on Software Rasterizer fallback (" + backend + ")!");
            showFallbackWarning();
        }

        // Apply sticky immersive fullscreen AFTER decor view is initialized
        hideSystemUI();
    }

    private void showFallbackWarning() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (isFinishing() || (Build.VERSION.SDK_INT >= Build.VERSION_CODES.JELLY_BEAN_MR1 && isDestroyed())) {
                    return;
                }
                new AlertDialog.Builder(RustCraftActivity.this)
                        .setTitle("⚠️ Performance Warning")
                        .setMessage("Failed to initialize hardware graphics acceleration (OpenGL / GLES).\n\n"
                                + "The game has started in Software Rasterizer fallback mode. "
                                + "Performance on mobile devices will be significantly lower and framerate drops may occur.")
                        .setPositiveButton("OK", null)
                        .setIcon(android.R.drawable.ic_dialog_alert)
                        .setCancelable(true)
                        .show();

                Toast.makeText(
                        RustCraftActivity.this,
                        "RustCraft: Software Rasterizer active (reduced performance)",
                        Toast.LENGTH_LONG
                ).show();
            }
        });
    }

    private void hideSystemUI() {
        if (getWindow() == null || getWindow().peekDecorView() == null) {
            return;
        }
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

    // --- Soft Keyboard (IME) Management ---

    public void setKeyboardVisible(final boolean visible) {
        if (isKeyboardVisible == visible) {
            return;
        }
        isKeyboardVisible = visible;
        runOnUiThread(() -> {
            InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
            if (imm != null && surfaceView != null) {
                if (visible) {
                    surfaceView.requestFocus();
                    imm.showSoftInput(surfaceView, InputMethodManager.SHOW_FORCED);
                } else {
                    imm.hideSoftInputFromWindow(surfaceView.getWindowToken(), 0);
                    hideSystemUI();
                }
            }
        });
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
    public boolean dispatchTouchEvent(MotionEvent event) {
        onTouchEvent(event);
        return super.dispatchTouchEvent(event);
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        final int action = event.getActionMasked();
        final int pointerIndex = event.getActionIndex();
        final int pointerId = event.getPointerId(pointerIndex);
        final float viewW = surfaceView != null && surfaceView.getWidth() > 0 ? (float) surfaceView.getWidth() : (float) getResources().getDisplayMetrics().widthPixels;
        final float viewH = surfaceView != null && surfaceView.getHeight() > 0 ? (float) surfaceView.getHeight() : (float) getResources().getDisplayMetrics().heightPixels;

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
            if (isKeyboardVisible) {
                // If keyboard is active, back button dismisses chat
                setKeyboardVisible(false);
                RustCraftNative.nativeKeyEvent(111, true); // Escape
                RustCraftNative.nativeKeyEvent(111, false);
                return true;
            }
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
        if ((event.getSource() & InputDevice.SOURCE_JOYSTICK) == InputDevice.SOURCE_JOYSTICK
                || (event.getSource() & InputDevice.SOURCE_GAMEPAD) == InputDevice.SOURCE_GAMEPAD) {
            float lx = event.getAxisValue(MotionEvent.AXIS_X);
            float ly = event.getAxisValue(MotionEvent.AXIS_Y);
            float rx = event.getAxisValue(MotionEvent.AXIS_Z);
            float ry = event.getAxisValue(MotionEvent.AXIS_RZ);

            float lt = event.getAxisValue(MotionEvent.AXIS_LTRIGGER);
            if (lt == 0.0f) {
                lt = event.getAxisValue(MotionEvent.AXIS_BRAKE);
            }
            float rt = event.getAxisValue(MotionEvent.AXIS_RTRIGGER);
            if (rt == 0.0f) {
                rt = event.getAxisValue(MotionEvent.AXIS_GAS);
            }

            float hatX = event.getAxisValue(MotionEvent.AXIS_HAT_X);
            float hatY = event.getAxisValue(MotionEvent.AXIS_HAT_Y);

            RustCraftNative.nativeAxisEvent(lx, ly, rx, ry, lt, rt, hatX, hatY);
            return true;
        }
        return super.onGenericMotionEvent(event);
    }

    // --- Lifecycle ---

    @Override
    protected void onPause() {
        super.onPause();
        setKeyboardVisible(false);
        RustCraftNative.nativeSaveAndPause();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        RustCraftNative.nativeDestroy();
    }

    /**
     * Custom SurfaceView supporting full Android Software Keyboard (IME) input.
     */
    private class GameSurfaceView extends SurfaceView {
        public GameSurfaceView(Context context) {
            super(context);
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            RustCraftActivity.this.onTouchEvent(event);
            return true;
        }

        @Override
        public boolean onCheckIsTextEditor() {
            return true;
        }

        @Override
        public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
            outAttrs.actionLabel = null;
            outAttrs.inputType = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS;
            outAttrs.imeOptions = EditorInfo.IME_ACTION_DONE | EditorInfo.IME_FLAG_NO_FULLSCREEN;
            return new BaseInputConnection(this, false) {
                @Override
                public boolean commitText(CharSequence text, int newCursorPosition) {
                    if (text != null && text.length() > 0) {
                        RustCraftNative.nativeTextInput(text.toString());
                    }
                    return true;
                }

                @Override
                public boolean deleteSurroundingText(int beforeLength, int afterLength) {
                    for (int i = 0; i < beforeLength; i++) {
                        RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_DEL, true);
                        RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_DEL, false);
                    }
                    return true;
                }

                @Override
                public boolean sendKeyEvent(KeyEvent event) {
                    if (event.getAction() == KeyEvent.ACTION_DOWN) {
                        if (event.getKeyCode() == KeyEvent.KEYCODE_ENTER) {
                            RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_ENTER, true);
                            RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_ENTER, false);
                            setKeyboardVisible(false);
                            return true;
                        } else if (event.getKeyCode() == KeyEvent.KEYCODE_DEL) {
                            RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_DEL, true);
                            RustCraftNative.nativeKeyEvent(KeyEvent.KEYCODE_DEL, false);
                            return true;
                        }
                    }
                    return super.sendKeyEvent(event);
                }
            };
        }
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

                    int flags = RustCraftNative.nativeStep(dt);

                    // Check if chat prompt is open (Bit 0)
                    boolean chatActive = (flags & 1) != 0;
                    if (chatActive != isKeyboardVisible) {
                        setKeyboardVisible(chatActive);
                    }

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
