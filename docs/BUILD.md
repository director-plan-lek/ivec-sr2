# บิลด์ Alux ทุกแพลตฟอร์ม

## สรุปสั้น

| เป้าหมาย | ได้ไฟล์ | ต้องมีอะไร | ทำบนเครื่องนี้ได้ไหม |
| --- | --- | --- | --- |
| Windows | `Alux.exe` + `Alux_0.1.0_x64-setup.exe` | Rust + Node | ✅ ได้ |
| Linux | `alux_0.1.0_amd64.deb`, `.AppImage` | Rust + Node + webkit2gtk (บน Linux) | ❌ ต้องทำบน Linux/WSL/CI |
| Android | `alux-0.1.0.apk`, `.aab` | Rust + Node + **JDK 17** + **Android SDK/NDK** | ⚠️ ยังขาด JDK/SDK |
| iOS | `Alux.ipa` | **macOS + Xcode** + Apple Developer Program | ❌ ทำบน Windows ไม่ได้ |

---

## 1) Windows — พร้อมแล้ว

```powershell
npx tauri build
```

ได้:
- `src-tauri/target/release/alux.exe` (เปิดได้เลย แบบ portable)
- `src-tauri/target/release/bundle/nsis/Alux_0.1.0_x64-setup.exe` (ตัวติดตั้ง)

ข้อกำหนดของเครื่องปลายทาง: Windows 10/11 (มี WebView2 มาให้แล้ว)

ถ้าต้องการไฟล์ .msi ให้เปลี่ยน `bundle.targets` ใน `src-tauri/tauri.conf.json`
เป็น `["nsis", "msi"]` (ต้องมี WiX ซึ่ง tauri จะดาวน์โหลดให้)

---

## 2) Linux

ทำบนเครื่อง Linux (หรือ WSL2 + WSLg):

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
npm install
npx tauri build
```

ได้ `.deb` และ `.AppImage` ใน `src-tauri/target/release/bundle/`

ทางเลือกที่ไม่ต้องมีเครื่อง Linux: ใช้ GitHub Actions (`.github/workflows build matrix`
หรือ `tauri-action`) สร้างทั้ง Windows/Linux/macOS พร้อมกัน

---

## 3) Android (.apk / .aab)

ต้องติดตั้ง (ยังไม่มีในเครื่องนี้):

1. **JDK 17** — https://adoptium.net/temurin/releases/?version=17
2. **Android Studio** (ให้ SDK + Platform 34 + Build-Tools)
3. **NDK 26.x** และ `cargo-ndk`

```bash
# หลังตั้งค่า ANDROID_HOME / NDK_HOME / JAVA_HOME แล้ว
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
cargo install cargo-ndk
npm install
npx tauri android init
npx tauri android build
```

ได้ `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`
(เปลี่ยนชื่อไฟล์เป็น `Alux-0.1.0.apk` ได้เลย)

หมายเหตุที่ต้องรู้ก่อนแจก:
- การติดตั้ง .apk นอก Play Store ต้องเปิด "ติดตั้งจากแหล่งที่ไม่รู้จัก"
- Android System WebView คือ engine ของแท็บ — Google Docs ใช้ได้
- การล็อกอินบัญชี Google ใน WebView อาจถูกปฏิเสธ ("browser not secure")
  Alux แก้ด้วยการตั้ง User-Agent ให้เป็น Chrome/Edge ปกติ ซึ่งช่วยได้ในหลายกรณี
  แต่ Google เปลี่ยนนโยบายได้ทุกเมื่อ — ทางที่ชัวร์คือเปิด OAuth ใน Chrome Custom Tabs
  (ต้องเพิ่มปลั๊กอินแยก)

---

## 4) iOS (.ipa)

**จำเป็นต้องมี Mac** (Xcode + iOS SDK) — Apple ไม่ให้คอมไพล์ iOS บน Windows/Linux

```bash
# บน macOS
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
npm install
npx tauri ios init
npx tauri ios build          # หรือเปิด Xcode ผ่าน `npx tauri ios open`
```

แจกจ่าย:
- **App Store**: ต้องมี Apple Developer Program (99 USD/ปี) + App Review
- **TestFlight / Ad Hoc**: ต้องมี UDID เครื่องผู้รับ (จำกัด 100 เครื่อง/ปี)
- Apple บังคับให้เบราว์เซอร์ใน App Store ใช้ WebKit เท่านั้น → แผน WebView ของ Alux
  จึงถูกต้องตามกฎอยู่แล้ว

---

## เปลี่ยนชื่อ/ไอคอน

- ชื่อที่แสดง: `productName` ใน `src-tauri/tauri.conf.json`
- ชื่อไฟล์ exe: เพิ่ม `"mainBinaryName": "Alux"` ใน `tauri.conf.json` (ถ้าต้องการ)
- ไอคอน: แก้ `scripts/make_icon.py` แล้วรัน

```powershell
python scripts\make_icon.py
npx tauri icon .\app-icon.png
```

- Package id: `identifier` (`com.alux.browser`) — ต้องไม่ซ้ำใครบน Play Store / App Store
