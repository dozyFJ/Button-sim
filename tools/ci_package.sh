#!/usr/bin/env bash
set -euo pipefail
export OTA_SHELL_HASH="$(python3 tools/shell_hash.py game)"
export OTA_SEQUENCE="$((GITHUB_RUN_ID * 100 + GITHUB_RUN_ATTEMPT))"
gradle -p android-shell --no-daemon assembleRelease
cp android-shell/app/build/outputs/apk/release/app-release.apk build/button-sim-release.apk
python3 tools/check_shell_apk.py game build/button-sim-release.apk
"$ANDROID_HOME/build-tools/35.0.0/apksigner" verify --verbose --print-certs build/button-sim-release.apk
"$ANDROID_HOME/build-tools/35.0.0/zipalign" -c -P 16 4 build/button-sim-release.apk
python3 tools/build_ota.py --kind game --input android-shell/app/src/main/jniLibs/arm64-v8a --output build/game-ota --build "$GITHUB_SHA" --sequence "$OTA_SEQUENCE" --keystore "$BUTTON_SIM_KEYSTORE" --shell-hash "$OTA_SHELL_HASH" --nm "$ANDROID_HOME/ndk/27.2.12479018/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-nm"
bash tools/check_ota.sh game build/game-ota
ls -lh build/button-sim-release.apk build/game-ota/*

