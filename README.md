# Button Sim

Landscape Android clicker with a compact top header, integrated navigation, a cartoon ground field, 3D pets and team auto attacks.

Download **button-sim-release.apk** from [latest-apk](https://github.com/dozyFJ/Button-sim/releases/tag/latest-apk).
Install the native OTA shell once, then use **Pause → Обновить игру (OTA)** to download compatible game cores and restart.

Release builds optimize size and strip debug symbols. APK and OTA use one permanent private key, stored in repository Actions Secrets:

- `BUTTON_SIM_KEYSTORE_BASE64`: base64 PKCS12 keystore, alias `androiddebugkey`.
- `BUTTON_SIM_KEYSTORE_PASSWORD`: keystore and key password.

Without secrets CI produces unsigned validation artifacts only and skips publication.
Previous CI debug builds used different random keys. A new signature cannot update an APK signed with a lost old key; do not uninstall an existing copy without accounting for saved progress.

See [OTA design and operations](docs/OTA.md).
