// RustCraft WASM Frontend Glue & Input Handler
import init, { WebGame } from './pkg/rustcraft.js';

let game = null;
let wasmMemory = null;
let canvasCtx = null;
let sharedImageData = null;

const canvas = document.getElementById('game-canvas');
const canvasContainer = document.getElementById('canvas-container');
const loadingOverlay = document.getElementById('loading-overlay');
const pointerlockPrompt = document.getElementById('pointerlock-prompt');
const fpsText = document.getElementById('fps-text');
const fullscreenBtn = document.getElementById('fullscreen-btn');
const gamepadDot = document.getElementById('gamepad-dot');
const gamepadName = document.getElementById('gamepad-name');

// Keyboard input state
const keys = {
  w: false, s: false, a: false, d: false,
  space: false, shift: false, ctrl: false, c: false,
  escape: false, p: false, f: false, e: false, q: false, g: false, m: false, f5: false,
  enter: false, up: false, down: false, left: false, right: false,
  n: false, del: false,
  digit1: false, digit2: false, digit3: false, digit4: false, digit5: false,
  digit6: false, digit7: false, digit8: false, digit9: false, digit0: false,
};

// Mouse & cursor state
let mousePos = { x: 0, y: 0, valid: false };
let mouseDelta = { x: 0, y: 0 };
let mouseButtons = { left: false, right: false };
let scrollDelta = 0;
let isPointerLocked = false;
let justLocked = false;
let isGameCurrentlyPlaying = false;

// Gamepad button edge detection (just pressed)
let prevGpButtons = new Array(20).fill(false);

function updateCanvasMousePos(evt) {
  const rect = canvas.getBoundingClientRect();
  if (rect.width > 0 && rect.height > 0) {
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    mousePos.x = (evt.clientX - rect.left) * scaleX;
    mousePos.y = (evt.clientY - rect.top) * scaleY;
    mousePos.valid = true;
  }
}

// Keyboard event listeners
window.addEventListener('keydown', (e) => {
  const code = e.code;
  if (code === 'KeyW') keys.w = true;
  else if (code === 'KeyS') keys.s = true;
  else if (code === 'KeyA') keys.a = true;
  else if (code === 'KeyD') keys.d = true;
  else if (code === 'Space') { keys.space = true; e.preventDefault(); }
  else if (code === 'ShiftLeft' || code === 'ShiftRight') keys.shift = true;
  else if (code === 'ControlLeft' || code === 'ControlRight') keys.ctrl = true;
  else if (code === 'KeyC') keys.c = true;
  else if (code === 'Escape') keys.escape = true;
  else if (code === 'KeyP') keys.p = true;
  else if (code === 'KeyF') keys.f = true;
  else if (code === 'KeyE') keys.e = true;
  else if (code === 'KeyQ') keys.q = true;
  else if (code === 'KeyG') keys.g = true;
  else if (code === 'KeyM') keys.m = true;
  else if (code === 'F5') { keys.f5 = true; e.preventDefault(); }
  else if (code === 'Enter') keys.enter = true;
  else if (code === 'ArrowUp') { keys.up = true; e.preventDefault(); }
  else if (code === 'ArrowDown') { keys.down = true; e.preventDefault(); }
  else if (code === 'ArrowLeft') { keys.left = true; e.preventDefault(); }
  else if (code === 'ArrowRight') { keys.right = true; e.preventDefault(); }
  else if (code === 'KeyN') keys.n = true;
  else if (code === 'Delete' || code === 'Backspace') keys.del = true;
  else if (code === 'Digit1') keys.digit1 = true;
  else if (code === 'Digit2') keys.digit2 = true;
  else if (code === 'Digit3') keys.digit3 = true;
  else if (code === 'Digit4') keys.digit4 = true;
  else if (code === 'Digit5') keys.digit5 = true;
  else if (code === 'Digit6') keys.digit6 = true;
  else if (code === 'Digit7') keys.digit7 = true;
  else if (code === 'Digit8') keys.digit8 = true;
  else if (code === 'Digit9') keys.digit9 = true;
  else if (code === 'Digit0') keys.digit0 = true;
});

window.addEventListener('keyup', (e) => {
  const code = e.code;
  if (code === 'KeyW') keys.w = false;
  else if (code === 'KeyS') keys.s = false;
  else if (code === 'KeyA') keys.a = false;
  else if (code === 'KeyD') keys.d = false;
  else if (code === 'Space') keys.space = false;
  else if (code === 'ShiftLeft' || code === 'ShiftRight') keys.shift = false;
  else if (code === 'ControlLeft' || code === 'ControlRight') keys.ctrl = false;
  else if (code === 'KeyC') keys.c = false;
  else if (code === 'Escape') keys.escape = false;
  else if (code === 'KeyP') keys.p = false;
  else if (code === 'KeyF') keys.f = false;
  else if (code === 'KeyE') keys.e = false;
  else if (code === 'KeyQ') keys.q = false;
  else if (code === 'KeyG') keys.g = false;
  else if (code === 'KeyM') keys.m = false;
  else if (code === 'F5') keys.f5 = false;
  else if (code === 'Enter') keys.enter = false;
  else if (code === 'ArrowUp') keys.up = false;
  else if (code === 'ArrowDown') keys.down = false;
  else if (code === 'ArrowLeft') keys.left = false;
  else if (code === 'ArrowRight') keys.right = false;
  else if (code === 'KeyN') keys.n = false;
  else if (code === 'Delete' || code === 'Backspace') keys.del = false;
  else if (code === 'Digit1') keys.digit1 = false;
  else if (code === 'Digit2') keys.digit2 = false;
  else if (code === 'Digit3') keys.digit3 = false;
  else if (code === 'Digit4') keys.digit4 = false;
  else if (code === 'Digit5') keys.digit5 = false;
  else if (code === 'Digit6') keys.digit6 = false;
  else if (code === 'Digit7') keys.digit7 = false;
  else if (code === 'Digit8') keys.digit8 = false;
  else if (code === 'Digit9') keys.digit9 = false;
  else if (code === 'Digit0') keys.digit0 = false;
});

// Mouse event listeners
window.addEventListener('mousemove', (e) => {
  updateCanvasMousePos(e);
  if (isPointerLocked) {
    if (justLocked) {
      justLocked = false;
      return;
    }
    // Clamp anomalous delta spikes on focus transition
    const dx = Math.max(-80, Math.min(80, e.movementX || 0));
    const dy = Math.max(-80, Math.min(80, e.movementY || 0));
    mouseDelta.x += dx;
    mouseDelta.y += dy;
  }
});

canvas.addEventListener('mousedown', (e) => {
  updateCanvasMousePos(e);
  if (e.button === 0) mouseButtons.left = true;
  if (e.button === 2) mouseButtons.right = true;

  // Capture cursor ONLY when in active gameplay (Playing), not in Menu or Inventory
  if (isGameCurrentlyPlaying && !isPointerLocked && document.pointerLockElement !== canvas) {
    canvas.requestPointerLock().catch(() => {});
  }
});

window.addEventListener('mouseup', (e) => {
  if (e.button === 0) mouseButtons.left = false;
  if (e.button === 2) mouseButtons.right = false;
});

canvas.addEventListener('contextmenu', (e) => e.preventDefault());

canvas.addEventListener('wheel', (e) => {
  scrollDelta += e.deltaY < 0 ? 1 : -1;
  e.preventDefault();
}, { passive: false });

// Pointer Lock Listener
document.addEventListener('pointerlockchange', () => {
  const lockedNow = (document.pointerLockElement === canvas);
  if (lockedNow && !isPointerLocked) {
    justLocked = true;
  }
  isPointerLocked = lockedNow;
  if (isPointerLocked) {
    pointerlockPrompt.classList.add('hidden');
  }
});

// Fullscreen
fullscreenBtn.addEventListener('click', () => {
  if (!document.fullscreenElement) {
    canvasContainer.requestFullscreen().catch(() => {});
  } else {
    document.exitFullscreen().catch(() => {});
  }
});

// Poll Gamepad via HTML5 Gamepad API
function pollGamepad() {
  const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
  let gp = null;
  for (let i = 0; i < gamepads.length; i++) {
    if (gamepads[i]) {
      gp = gamepads[i];
      break;
    }
  }

  if (gp) {
    gamepadDot.classList.add('connected');
    gamepadName.textContent = gp.id.substring(0, 32);

    const getBtn = (idx) => gp.buttons[idx] ? gp.buttons[idx].pressed : false;
    const getAxis = (idx) => {
      const val = gp.axes[idx] || 0.0;
      return Math.abs(val) > 0.15 ? val : 0.0;
    };

    const bSouth = getBtn(0);
    const bEast = getBtn(1);
    const bWest = getBtn(2);
    const bNorth = getBtn(3);
    const bLb = getBtn(4);
    const bRb = getBtn(5);
    const bLt = gp.buttons[6] ? (gp.buttons[6].value || (gp.buttons[6].pressed ? 1.0 : 0.0)) : 0.0;
    const bRt = gp.buttons[7] ? (gp.buttons[7].value || (gp.buttons[7].pressed ? 1.0 : 0.0)) : 0.0;
    const bSelect = getBtn(8);
    const bStart = getBtn(9);
    const bL3 = getBtn(10);
    const bR3 = getBtn(11);
    const dUp = getBtn(12);
    const dDown = getBtn(13);
    const dLeft = getBtn(14);
    const dRight = getBtn(15);

    const currentBtns = [bSouth, bEast, bWest, bNorth, bLb, bRb, bLt > 0.3, bRt > 0.3, bSelect, bStart, bL3, bR3, dUp, dDown, dLeft, dRight];
    const justPressed = currentBtns.map((val, idx) => val && !prevGpButtons[idx]);
    prevGpButtons = currentBtns;

    return {
      lx: getAxis(0),
      ly: -getAxis(1),
      rx: getAxis(2),
      ry: getAxis(3),
      lt: bLt,
      rt: bRt,
      btnSouth: bSouth,
      btnEast: bEast,
      btnWest: bWest,
      btnNorth: bNorth,
      btnLb: bLb,
      btnRb: bRb,
      btnL3: bL3,
      btnR3: bR3,
      btnStart: bStart,
      btnSelect: bSelect,
      dpadUp: dUp,
      dpadDown: dDown,
      dpadLeft: dLeft,
      dpadRight: dRight,
      southJust: justPressed[0],
      eastJust: justPressed[1],
      westJust: justPressed[2],
      northJust: justPressed[3],
      lbJust: justPressed[4],
      rbJust: justPressed[5],
      ltJust: justPressed[6],
      rtJust: justPressed[7],
      selectJust: justPressed[8],
      startJust: justPressed[9],
      r3Just: justPressed[11],
      dpadUpJust: justPressed[12],
      dpadDownJust: justPressed[13],
      dpadLeftJust: justPressed[14],
      dpadRightJust: justPressed[15],
    };
  } else {
    gamepadDot.classList.remove('connected');
    gamepadName.textContent = 'No controller connected';
    return {
      lx: 0, ly: 0, rx: 0, ry: 0, lt: 0, rt: 0,
      btnSouth: false, btnEast: false, btnWest: false, btnNorth: false,
      btnLb: false, btnRb: false, btnL3: false, btnR3: false,
      btnStart: false, btnSelect: false,
      dpadUp: false, dpadDown: false, dpadLeft: false, dpadRight: false,
      southJust: false, eastJust: false, westJust: false, northJust: false,
      lbJust: false, rbJust: false, ltJust: false, rtJust: false,
      selectJust: false, startJust: false, r3Just: false,
      dpadUpJust: false, dpadDownJust: false, dpadLeftJust: false, dpadRightJust: false,
    };
  }
}

// Main rendering & animation loop
let lastTime = performance.now();
let fpsFrames = 0;
let fpsTimer = performance.now();

function frameLoop(now) {
  const dt = Math.min((now - lastTime) / 1000.0, 0.1);
  lastTime = now;

  const gp = pollGamepad();

  if (game) {
    try {
      const statusStr = game.update_and_render(
        dt,
        mouseDelta.x,
        mouseDelta.y,
        mousePos.x,
        mousePos.y,
        mousePos.valid,
        mouseButtons.left,
        mouseButtons.right,
        scrollDelta,
        isPointerLocked,
        keys.w, keys.s, keys.a, keys.d,
        keys.space, keys.shift, keys.ctrl, keys.c,
        keys.escape, keys.p, keys.f, keys.e, keys.q, keys.g, keys.m, keys.f5,
        keys.enter, keys.up, keys.down, keys.left, keys.right,
        keys.n, keys.del,
        keys.digit1, keys.digit2, keys.digit3, keys.digit4, keys.digit5,
        keys.digit6, keys.digit7, keys.digit8, keys.digit9, keys.digit0,
        gp.lx, gp.ly, gp.rx, gp.ry, gp.lt, gp.rt,
        gp.btnSouth, gp.btnEast, gp.btnWest, gp.btnNorth,
        gp.btnLb, gp.btnRb, gp.btnL3, gp.btnR3,
        gp.btnStart, gp.btnSelect,
        gp.dpadUp, gp.dpadDown, gp.dpadLeft, gp.dpadRight,
        gp.southJust, gp.eastJust, gp.westJust, gp.northJust,
        gp.lbJust, gp.rbJust, gp.r3Just,
        gp.startJust, gp.selectJust,
        gp.dpadUpJust, gp.dpadDownJust, gp.dpadLeftJust, gp.dpadRightJust,
        gp.rtJust, gp.ltJust,
      );

      isGameCurrentlyPlaying = statusStr.includes('playing');

      // Direct zero-copy pixel buffer blitting from WASM memory
      if (wasmMemory && sharedImageData && canvasCtx) {
        const ptr = game.get_rgba_ptr();
        const wasmView = new Uint8Array(wasmMemory.buffer, ptr, 960 * 540 * 4);
        sharedImageData.data.set(wasmView);
        canvasCtx.putImageData(sharedImageData, 0, 0);
      }

      // Update pointer lock prompt visibility
      if (isGameCurrentlyPlaying && !isPointerLocked) {
        pointerlockPrompt.classList.remove('hidden');
      } else {
        pointerlockPrompt.classList.add('hidden');
      }

      if (statusStr.includes('release_mouse') && isPointerLocked) {
        document.exitPointerLock();
      }
    } catch (err) {
      console.error("Error in RustCraft WASM game loop:", err);
    }
  }

  // Reset per-frame deltas
  mouseDelta.x = 0;
  mouseDelta.y = 0;
  scrollDelta = 0;

  // FPS Counter
  fpsFrames++;
  if (now - fpsTimer >= 500) {
    const currentFps = Math.round((fpsFrames * 1000) / (now - fpsTimer));
    fpsText.textContent = `${currentFps} FPS`;
    fpsFrames = 0;
    fpsTimer = now;
  }

  requestAnimationFrame(frameLoop);
}

// Initialize WebAssembly module
async function start() {
  try {
    const wasmExports = await init();
    wasmMemory = wasmExports.memory;

    canvasCtx = canvas.getContext('2d', { alpha: false, desynchronized: true });
    sharedImageData = canvasCtx.createImageData(960, 540);

    game = new WebGame('game-canvas', 1337);
    loadingOverlay.classList.remove('active');
    requestAnimationFrame(frameLoop);
  } catch (err) {
    loadingOverlay.innerHTML = `
      <h2 style="color: #ef4444;">WebAssembly Initialization Error</h2>
      <p style="color: #fca5a5; max-width: 500px; text-align: center;">${err}</p>
    `;
    console.error("WASM initialization failed:", err);
  }
}

start();
