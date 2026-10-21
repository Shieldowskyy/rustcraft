#!/usr/bin/env bash
set -e

# ============================================================================
# Build script for RustCraft on Android (ARM64 / aarch64)
# Produces both librustcraft.so and a signed, installable RustCraft.apk
# ============================================================================

export PATH="$HOME/.cargo/bin:$PATH"

echo "=== Building RustCraft for Android (ARM64) ==="

# ----------------------------------------------------------------------------
# 1. Locate Android SDK
# ----------------------------------------------------------------------------
SDK_CANDIDATES=(
    "$ANDROID_HOME"
    "$ANDROID_SDK_ROOT"
    "$HOME/Android/Sdk"
    "/opt/android-sdk"
    "/usr/lib/android-sdk"
)

ANDROID_SDK=""
for path in "${SDK_CANDIDATES[@]}"; do
    if [ -n "$path" ] && [ -d "$path" ]; then
        ANDROID_SDK="$path"
        break
    fi
done

if [ -z "$ANDROID_SDK" ]; then
    echo "==> Error: Android SDK not found. Set \$ANDROID_HOME or install to ~/Android/Sdk"
    exit 1
fi
echo "✓ Android SDK located: $ANDROID_SDK"

# ----------------------------------------------------------------------------
# 2. Locate Android NDK
# ----------------------------------------------------------------------------
NDK_CANDIDATES=(
    "$ANDROID_NDK_HOME"
    "$ANDROID_NDK_ROOT"
    $(ls -d "$ANDROID_SDK/ndk/"* 2>/dev/null | sort -V | tail -n 1)
    "/opt/android-ndk"
)

ANDROID_NDK=""
for path in "${NDK_CANDIDATES[@]}"; do
    if [ -n "$path" ] && [ -d "$path" ]; then
        ANDROID_NDK="$path"
        break
    fi
done

if [ -z "$ANDROID_NDK" ]; then
    echo "==> Error: Android NDK not found in $ANDROID_SDK/ndk. Please install via SDK manager."
    exit 1
fi
echo "✓ Android NDK located: $ANDROID_NDK"

# Find NDK LLVM toolchain
NDK_LLVM="$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64"
if [ ! -d "$NDK_LLVM" ]; then
    NDK_LLVM=$(ls -d "$ANDROID_NDK/toolchains/llvm/prebuilt/"* 2>/dev/null | head -n 1)
fi

NDK_CLANG=$(ls "$NDK_LLVM/bin/aarch64-linux-android"*"-clang" 2>/dev/null | sort -V | head -n 1)
NDK_STRIP="$NDK_LLVM/bin/llvm-strip"

if [ ! -f "$NDK_CLANG" ]; then
    echo "==> Error: aarch64 NDK clang not found in $NDK_LLVM/bin"
    exit 1
fi
echo "✓ NDK Clang cross-compiler: $(basename "$NDK_CLANG")"

# ----------------------------------------------------------------------------
# 3. Locate Java Compiler (javac) & Keytool
# ----------------------------------------------------------------------------
JAVA_BIN=""
if command -v javac &> /dev/null; then
    JAVA_BIN="$(dirname "$(command -v javac)")"
else
    JBR_CANDIDATES=(
        "$HOME/android-studio/jbr/bin"
        $(ls -d "$HOME/.local/share/JetBrains/Toolbox/apps/android-studio/"*/jbr/bin 2>/dev/null | head -n 1)
        $(ls -d "$HOME/Pobrane/jdk-"*/bin 2>/dev/null | head -n 1)
        "/usr/lib/jvm/java-21-openjdk/bin"
        "/usr/lib/jvm/java-17-openjdk/bin"
    )
    for path in "${JBR_CANDIDATES[@]}"; do
        if [ -x "$path/javac" ]; then
            JAVA_BIN="$path"
            break
        fi
    done
fi

if [ -z "$JAVA_BIN" ]; then
    echo "==> Error: Java compiler (javac) not found. Please install OpenJDK."
    exit 1
fi
export PATH="$JAVA_BIN:$PATH"
echo "✓ Java toolchain located: $JAVA_BIN"

# ----------------------------------------------------------------------------
# 4. Locate Android Build-Tools & android.jar
# ----------------------------------------------------------------------------
BUILD_TOOLS_DIR=$(ls -d "$ANDROID_SDK/build-tools/"* 2>/dev/null | sort -V | tail -n 1)
AAPT2="$BUILD_TOOLS_DIR/aapt2"
D8="$BUILD_TOOLS_DIR/d8"
ZIPALIGN="$BUILD_TOOLS_DIR/zipalign"
APKSIGNER="$BUILD_TOOLS_DIR/apksigner"

PLATFORM_DIR=$(ls -d "$ANDROID_SDK/platforms/android-"* 2>/dev/null | sort -V | tail -n 1)
ANDROID_JAR="$PLATFORM_DIR/android.jar"

if [ ! -f "$AAPT2" ] || [ ! -f "$ANDROID_JAR" ]; then
    echo "==> Error: Missing build-tools or platforms in Android SDK."
    exit 1
fi
echo "✓ Android build-tools: $(basename "$BUILD_TOOLS_DIR")"
echo "✓ Target platform: $(basename "$PLATFORM_DIR")"

# ----------------------------------------------------------------------------
# 5. Check Rust target
# ----------------------------------------------------------------------------
TARGET="aarch64-linux-android"
if command -v rustup &> /dev/null; then
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo "==> Adding target $TARGET via rustup..."
        rustup target add "$TARGET"
    fi
fi

# ----------------------------------------------------------------------------
# 6. Compile Native Shared Library (librustcraft.so)
# ----------------------------------------------------------------------------
echo "==> Compiling native engine (Cargo release build for $TARGET)..."
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_CLANG"
export RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"

cargo build --release --target "$TARGET" --lib

SO_PATH="target/$TARGET/release/librustcraft.so"
if [ ! -f "$SO_PATH" ]; then
    echo "==> Error: Native library $SO_PATH was not created."
    exit 1
fi

echo -n "✓ Native library compiled. Size before strip: "
du -h "$SO_PATH" | awk '{print $1}'

if [ -f "$NDK_STRIP" ]; then
    "$NDK_STRIP" "$SO_PATH"
    echo -n "✓ Symbols stripped. Size after strip: "
    du -h "$SO_PATH" | awk '{print $1}'
fi

# Copy into Android project jniLibs
JNILIBS_DIR="android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$JNILIBS_DIR"
cp -f "$SO_PATH" "$JNILIBS_DIR/librustcraft.so"

# ----------------------------------------------------------------------------
# 7. Package and Sign APK
# ----------------------------------------------------------------------------
echo "==> Packaging Android APK (RustCraft.apk)..."
WORK_DIR="target/android_build"
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR/gen" "$WORK_DIR/classes" "$WORK_DIR/dex"

# 7a. Compile and Link Android Resources
"$AAPT2" compile --dir android/app/src/main/res -o "$WORK_DIR/res.zip"
"$AAPT2" link \
    -I "$ANDROID_JAR" \
    --manifest android/app/src/main/AndroidManifest.xml \
    -o "$WORK_DIR/unaligned.apk" \
    "$WORK_DIR/res.zip" \
    --java "$WORK_DIR/gen" \
    --auto-add-overlay

# 7b. Compile Java Sources
javac -source 1.8 -target 1.8 \
    -cp "$ANDROID_JAR" \
    -d "$WORK_DIR/classes" \
    "$WORK_DIR/gen/com/rustcraft/game/R.java" \
    android/app/src/main/java/com/rustcraft/game/*.java

# 7c. Dex Bytecode
"$D8" --lib "$ANDROID_JAR" --output "$WORK_DIR/dex" "$WORK_DIR/classes/com/rustcraft/game/"*.class

# 7d. Package classes.dex and librustcraft.so into APK
python3 - << EOF
import zipfile, shutil
shutil.copy("$WORK_DIR/unaligned.apk", "$WORK_DIR/combined.apk")
with zipfile.ZipFile("$WORK_DIR/combined.apk", "a", compression=zipfile.ZIP_DEFLATED) as z:
    z.write("$WORK_DIR/dex/classes.dex", "classes.dex")
    z.write("$JNILIBS_DIR/librustcraft.so", "lib/arm64-v8a/librustcraft.so")
EOF

# 7e. Zipalign
"$ZIPALIGN" -f -p 4 "$WORK_DIR/combined.apk" "$WORK_DIR/aligned.apk"

# 7f. Ensure Debug Keystore Exists
KEYSTORE="$HOME/.android/debug.keystore"
if [ ! -f "$KEYSTORE" ]; then
    echo "==> Creating debug signing key..."
    mkdir -p "$HOME/.android"
    keytool -genkey -v -keystore "$KEYSTORE" -storepass android -alias androiddebugkey \
        -keypass android -keyalg RSA -keysize 2048 -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US"
fi

# 7g. Sign APK
"$APKSIGNER" sign \
    --ks "$KEYSTORE" \
    --ks-pass pass:android \
    --key-pass pass:android \
    --out "RustCraft.apk" \
    "$WORK_DIR/aligned.apk"

"$APKSIGNER" verify "RustCraft.apk"

echo "--------------------------------------------------"
echo " SUCCESS! Android APK built and signed successfully."
echo -n " Final APK Size: "
du -h "RustCraft.apk" | awk '{print $1}'
echo " Output path:    $(pwd)/RustCraft.apk"
echo "--------------------------------------------------"

# If an ADB device is connected, prompt or auto-install
if command -v adb &> /dev/null; then
    DEVICE_COUNT=$(adb devices | grep -v "List of devices" | grep -v "^$" | wc -l)
    if [ "$DEVICE_COUNT" -gt 0 ]; then
        echo "✓ Connected Android device detected! Installing..."
        adb install -r "RustCraft.apk"
        echo "✓ RustCraft installed on device! Launch with:"
        echo "  adb shell am start -n com.rustcraft.game/.RustCraftActivity"
    else
        echo "To install on your phone/tablet via ADB:"
        echo "  adb install -r RustCraft.apk"
    fi
fi
