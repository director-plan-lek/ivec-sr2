# Alux — สถานะแต่ละแพลตฟอร์ม และวิธี build

Alux เขียนด้วย **TypeScript (React) + Tauri v2** ตัวเบราว์เซอร์จริงคือ webview ของระบบปฏิบัติการ
ดังนั้น "หน้าตา/ลอจิก" เขียนครั้งเดียว แต่ **ต้อง build บน OS ปลายทางแต่ละตัว** (Tauri ไม่ cross-compile ข้าม OS)

| แพลตฟอร์ม | สถานะ | ไฟล์ที่ได้ | ต้องมีอะไร |
|---|---|---|---|
| Windows 10/11 x64 | ✅ build + ทดสอบแล้ว | `alux.exe`, `*-setup.exe` | MSVC Build Tools + WebView2 (มีใน Win10/11) |
| Linux x64 | 🟡 config พร้อม (CI) | `.AppImage`, `.deb` | ต้อง build บน Linux เท่านั้น |
| Android | ✅ build ได้แล้ว (ยังไม่ทดสอบบนเครื่องจริง) | `.apk` 26.3 MB (arm64, debug) | JDK 17 + Android SDK 36 + NDK 26.1 |
| iOS | 🔴 ต้องมีเครื่อง Mac | `.app` / `.ipa` | macOS + Xcode + Apple Developer 99$/ปี |

---

## Windows (ทำเสร็จแล้ว)

```powershell
npm ci
npx tauri build
```

ได้ `src-tauri/target/release/alux.exe` (portable) และ
`src-tauri/target/release/bundle/nsis/Alux_0.1.0_x64-setup.exe` (ตัวติดตั้ง)

ข้อกำหนด: Visual Studio Build Tools (workload VCTools) + WebView2 Runtime
ถ้าติดตั้งแบบ per-machine (`installMode: "perMachine"`) ผู้ใช้ทั่วไปจะเขียนโฟลเดอร์โปรแกรมไม่ได้
จึงตั้งเป็น `currentUser` ไว้

## Linux

ต้อง build บน Linux (หรือ CI) เท่านั้น เพราะต้อง link กับ WebKitGTK ของระบบ

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libfuse2
npm ci
npx tauri build --bundles appimage,deb
```

ผลลัพธ์: `src-tauri/target/release/bundle/appimage/Alux_0.1.0_amd64.AppImage`
(ไฟล์เดียว รันได้ทุก distro ที่มี fuse — `chmod +x` แล้วดับเบิลคลิกได้)

หมายเหตุ: เครื่อง Windows นี้ build Linux ไม่ได้ (Docker Desktop ต้องใช้ WSL ซึ่งถูกปิด)
จึงเตรียมเป็น workflow CI ไว้แทน → `.github/workflows/build.yml` job `linux`

## Android (.apk)

> ✅ **build ได้แล้ว** — ได้ `release/Alux-0.1.0-arm64-debug.apk` (26.3 MB, arm64-v8a, debug-signed)
> ขั้นตอนทั้งหมด (รวม workaround เพราะ path มีช่องว่างและไม่มีสิทธิ์ symlink) อยู่ใน
> [`ANDROID-PORT.md`](ANDROID-PORT.md) — สถาปัตยกรรมแท็บบนมือถือใช้ "หนึ่งแท็บ = หนึ่งหน้าต่าง"
> เพราะ `add_child` เป็น desktop-only

```powershell
# 1) ติดตั้ง JDK 17 และ Android SDK + NDK ก่อน แล้วตั้ง env
$env:JAVA_HOME = "C:\Program Files\Eclipse Adoptium\jdk-17..."
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME = "$env:ANDROID_HOME\ndk\26.1.10909125"

# 2) เพิ่ม Rust target สำหรับมือถือ
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# 3) สร้างโปรเจกต์ Android แล้ว build
npx tauri android init
npx tauri android build --apk
```

- `--apk --debug` = ไฟล์ทดสอบติดตั้งได้เลย (unsigned/debug)
- ปล่อยให้คนอื่นติดตั้งจริงต้องมี **keystore** ของตัวเอง แล้วเซ็นด้วย `apksigner`
- ขนาดโดยประมาณ: ~8–12 MB (ไม่ต้องฝัง Chromium ใช้ WebView ของ Android)

## iOS — ข้อจำกัดที่ต้องรู้

**ทำจากเครื่อง Windows ไม่ได้เลย** ต้องมีเครื่อง Mac (หรือ CI ของ GitHub ที่เป็น macOS)

1. **ต้องมี macOS + Xcode** — ตัว compiler ของ Apple มีเฉพาะ macOS, `tauri ios build` เรียก `xcodebuild`
2. **ต้องมี Apple Developer Program 99 $/ปี** ถ้าจะลงเครื่องจริง/แจกจ่าย
   - ไม่มีบัญชี = build ได้แค่ simulator เปิดให้ใครใช้ไม่ได้
   - Free provisioning ลงได้เฉพาะเครื่องตัวเอง และใบรับรองหมดอายุใน 7 วัน
3. **ห้าม sideload แบบ Android** — iOS ไม่ให้ติดตั้งไฟล์ `.ipa` เอง ต้องผ่าน App Store หรือ TestFlight (สูงสุด 100 ผู้ทดสอบ, 90 วัน)
4. **App Store Review** — เบราว์เซอร์ที่ส่งเข้า App Store ต้องใช้ `WKWebView` (Alux ใช้อยู่แล้ว) แต่ Apple เข้มเรื่อง "แอปที่แค่ห่อเว็บไซต์" (Guideline 4.2) ต้องมีฟีเจอร์เฉพาะตัว
5. **WKWebView เข้มกว่า WebView2** — ไม่มี DRM (Netflix ไม่เล่น), service worker จำกัด, และเบราว์เซอร์อื่นบน iOS ก็ถูกบังคับให้ใช้ WebKit เหมือนกันหมด

สรุป: **ถ้าอาเล็กใช้ Android/Windows เป็นหลัก ไม่ต้องแตะ iOS เลย** ถ้าจำเป็นจริงควรจ้าง/เช่า Mac หรือใช้ Mac เครื่องเดียวในบ้าน build ครั้งเดียว

---

## ข้อจำกัดร่วมของสถาปัตยกรรม webview

- **DRM (Netflix, Spotify, Disney+)** เล่นไม่ได้ เพราะ webview ที่ฝังไม่มี Widevine license module
- เบราว์เซอร์ไม่ได้เป็นเจ้าของ engine เอง — ตามช่องโหว่/ฟีเจอร์ของ WebView2, WebKitGTK, WKWebView
- ส่วนขยาย (extensions) แบบ Chrome ไม่รองรับ
