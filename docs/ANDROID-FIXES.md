# IVEC SR 2 — บันทึกการแก้ไขสำหรับ Android

ไฟล์นี้รวบรวม **ทุกการแก้ไขที่ทำเพื่อให้แอปทำงานบน Android ได้จริง** เรียงตามอาการที่เจอ พร้อมสาเหตุและไฟล์ที่แก้
ใช้เป็นคู่มือเมื่อต้อง build เครื่องใหม่ หรือพอร์ตไปแพลตฟอร์มอื่น (ดู `IOS-PORT.md`)

> สถาปัตยกรรมย่อ: UI (React) รันในหน้าต่างหลัก ส่วนหน้าที่เปิดจากช่องค้นหา/ทางลัดจะเปิดเป็น
> **หน้าต่าง WebView แยก** เพราะ Tauri ทำ `Window::add_child` ได้เฉพาะเดสก์ท็อป

---

## 1. ติดตั้ง APK ไม่ผ่าน — "แพ็กเกจไม่ถูกต้อง"

| | |
|---|---|
| อาการ | Android ขึ้น `ไม่ได้ติดตั้งแอปเพราะดูเหมือนว่าแพ็กเกจจะไม่ถูกต้อง` |
| สาเหตุ | ไฟล์ APK ที่คัดลอกไปแจกถูกตัดหายกลางทาง (27,551,457 แทน 27,555,553 ไบต์) → `apksigner` ขึ้น `Missing META-INF/MANIFEST.MF` |
| วิธีแก้ | คัดลอกจาก `src-tauri/gen/android/app/build/outputs/apk/arm64/debug/app-arm64-debug.apk` โดยตรง และ **ตรวจ hash หลังคัดลอก** |
| ตรวจสอบ | `apksigner verify --verbose <apk>` ต้องขึ้น `Verifies` + `v2 scheme: true`<br>`zipalign -c -P 16 -v 4 <apk>` ต้องขึ้น `Verification successful` |

> ไฟล์ `.so` ใหญ่ 27 MB — ถ้าโอนผ่าน Wi-Fi/MTP ต้องอ่านกลับมาเทียบ SHA-256 ทุกครั้ง

---

## 2. แตะลิงก์ Google Docs แล้วขึ้น `ERR_UNKNOWN_URL_SCHEME`

| | |
|---|---|
| อาการ | กดลิงก์เอกสารแล้วขึ้นหน้า error `net::ERR_UNKNOWN_URL_SCHEME` ที่ URL เริ่มด้วย `intent://` |
| สาเหตุ | `RustWebViewClient.kt` (ไฟล์ที่ Tauri generate) ส่งทุก URL ให้ Rust ซึ่งไม่รู้จัก `intent://` แล้ว WebView พยายามโหลดเอง |
| ไฟล์ที่แก้ | `src-tauri/gen/android/app/src/main/java/com/alux/browser/generated/RustWebViewClient.kt` |
| วิธีแก้ | ใน `shouldOverrideUrlLoading` ดัก `intent://` → `Intent.parseUri(url, Intent.URI_INTENT_SCHEME)` แล้ว `startActivity` ; ถ้าไม่มีแอปปลายทางให้ใช้ `browser_fallback_url` |
| เพิ่มเติม | ส่ง `market://`, `tel:`, `mailto:`, `sms:`, `geo:`, `whatsapp:`, `line:` ให้ Android เปิดแทน (`NON_WEB_SCHEMES`) |
| เทมเพลต | `scripts/android/RustWebViewClient.kt` + `scripts/patch-android-webview.ps1` |

---

## 3. ล็อกอิน Google ไม่ผ่าน (`net::ERR_CONNECTION_ABORTED`)

| | |
|---|---|
| อาการ | หน้า `accounts.google.com/...SetSid...` ขึ้น `ERR_CONNECTION_ABORTED` หรือขึ้น "เกิดข้อผิดพลาดบางอย่าง" |
| สาเหตุ 1 | WebView ตั้งต้น **ปิดคุกกี้บุคคลที่สาม** สำหรับแอป targetSdk ≥ 30 แต่ flow ล็อกอิน Google ต้องใช้ |
| สาเหตุ 2 | User-Agent ของ WebView มีคำว่า `wv` → Google มองว่าเป็น WebView แล้วบล็อก |
| ไฟล์ที่แก้ | `generated/RustWebView.kt` (init) และ `src-tauri/src/lib.rs` (`.user_agent(chrome_ua())`) |
| วิธีแก้ | `CookieManager.getInstance().setAcceptCookie(true)` + `setAcceptThirdPartyCookies(this, true)` และตั้ง UA เป็น Chrome บน Android |

```kotlin
// RustWebView.kt — ใน init { }
CookieManager.getInstance().setAcceptCookie(true)
CookieManager.getInstance().setAcceptThirdPartyCookies(this, true)
```

---

## 4. ออกจากบัญชี Google ไม่ได้ (กด Drive แล้วเข้าเลย)

| | |
|---|---|
| อาการ | ล้างข้อมูลแล้วยังล็อกอินค้าง กด Drive แล้วเข้าไดรฟ์ทันที ไม่ถามรหัส |
| สาเหตุ | `clearAllBrowsingData()` ของ wry บน Android ลบแค่ cache/history/formdata — **ไม่ลบคุกกี้** |
| ไฟล์ที่แก้ | `generated/RustWebView.kt` (`clearAllBrowsingData`) |
| วิธีแก้ | เพิ่ม `removeAllCookies(null)`, `flush()`, `WebStorage.deleteAllData()`, `clearHttpAuthUsernamePassword()` |

ปุ่มในแอป: หน้าแรก → **"ล้างคุกกี้และออกจากบัญชี Google"**
คำสั่ง Rust: `clear_browsing_data` (`src-tauri/src/lib.rs`) → `webview.clear_all_browsing_data()` ทุก webview

---

## 5. ปุ่ม back ของเครื่องไม่ทำงาน

| | |
|---|---|
| อาการ | กด back แล้วไม่มีอะไรเกิดขึ้น |
| สาเหตุ | `TauriActivity.kt` ตั้ง `override val handleBackNavigation: Boolean = false` → ปิด `OnBackPressedCallback` ของ `WryActivity` ทั้งหมด |
| ไฟล์ที่แก้ | `app/src/main/java/com/alux/browser/MainActivity.kt` |
| วิธีแก้ | `override val handleBackNavigation: Boolean = true` → back จะเรียก `WebView.goBack()` เมื่อมีประวัติ และปิด activity เมื่อไม่มี |

---

## 6. แถบเครื่องมือของหน้าเว็บทับแถบเวลา/แบตเตอรี่

| | |
|---|---|
| อาการ | แถบเมนูของหน้าเว็บ (เช่น หน้าผลการค้นหา) อยู่ใต้แถบสถานะ กดไม่ได้ |
| สาเหตุ | **Android 15 (API 35+) บังคับ edge-to-edge** สำหรับแอป targetSdk ≥ 35 — ลบ `enableEdgeToEdge()` ไม่พอ |
| ไฟล์ที่แก้ | `MainActivity.kt` |
| วิธีแก้ | เติมระยะขอบตาม system bars ให้ content view |

```kotlin
val root = findViewById<View>(android.R.id.content)
ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
  val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
  view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
  WindowInsetsCompat.CONSUMED
}
```

---

## 7. ไม่มีแถบเครื่องมือบนหน้าที่เปิด (ไม่มีปุ่มย้อนกลับ/ปิด)

| | |
|---|---|
| สาเหตุ | หน้าเว็บเปิดเป็นหน้าต่างเต็มจอแยก ไม่มี chrome ของเบราว์เซอร์ให้กด |
| ไฟล์ที่แก้ | `src-tauri/src/lib.rs` → `#[cfg(mobile)] fn mobile_controls_js(label: &str)` |
| วิธีแก้ | ฉีดแถบแคปซูลลอยด้านล่าง (safe-area aware) ด้วย `.initialization_script(...)`:<br>**←** ย้อนกลับ · **→** ไปข้างหน้า · **⟳** โหลดใหม่ · **⌂** กลับหน้าหลัก |
| หมายเหตุ | สคริปต์ re-attach เองทุก 1.5 วินาที ถ้า SPA เขียน `body` ใหม่ |

---

## 8. ปุ่ม ⌂ กดแล้วไม่กลับหน้าหลัก

| | |
|---|---|
| สาเหตุ 1 | หน้าเว็บจากโดเมนภายนอก **เรียก IPC ไม่ได้** (`__TAURI_INTERNALS__` ถูกจำกัด) → `tab_close` เงียบ |
| สาเหตุ 2 | นำทางไป `alux://close` ตรง ๆ ไม่ได้ → `ERR_UNKNOWN_URL_SCHEME` |
| วิธีแก้ (3 ชั้น) | 1) `fetch('http://alux.localhost/close')` → Rust ปิดหน้าต่าง<br>2) ถ้าไม่สำเร็จ เปลี่ยนที่อยู่ไป `http://alux.localhost/close`<br>3) ถ้ายังไม่ปิด โหลด `http://tauri.localhost/` ในหน้าต่างเดิม → ได้หน้าแรกแน่นอน |
| ฝั่ง Rust | `register_uri_scheme_protocol("alux", ...)` ดัก path/host `close` → `get_webview_window(label)?.close()` และถ้าหาไม่เจอ จะปิดหน้าต่างที่กำลัง focus ที่ label เริ่มด้วย `tab-`/`app-` |

---

## 9. พิมพ์ IP ในวง LAN แล้วเปิดไม่ติด (หรือไปค้น Google)

| | |
|---|---|
| อาการ | พิมพ์ `192.168.1.39:8765` แล้วกลายเป็นค้นหา Google หรือเปิด `https://` แล้วเชื่อมต่อไม่ติด |
| สาเหตุ | `looks_like_host()` บังคับให้มีตัวอักษร → IPv4 ล้วนถูกมองเป็นคำค้นหา และทุก host ถูกเติม `https://` |
| ไฟล์ที่แก้ | `src-tauri/src/lib.rs` → `looks_like_host()`, `is_lan_host()`, `parse_url()` |
| วิธีแก้ | รับ IPv4 4 ออกเต็ตเป็น host และเติม `http://` ให้ host ในวง LAN (`localhost`, `*.local`, `*.lan`, 10.x, 127.x, 192.168.x, 172.16–31.x, 169.254.x) |

---

## 10. ค้นหาแล้วสปินเนอร์หมุนค้าง เปิดหน้าอื่นไม่ได้

| | |
|---|---|
| สาเหตุ | บน Android `tab_open` สร้างหน้าต่างใหม่ — invoke อาจค้างไม่ settle ทำให้สถานะ loading ไม่มีวันจบ |
| ไฟล์ที่แก้ | `src/App.tsx` (`go()`) |
| วิธีแก้ | watchdog 6 วินาที `setTimeout(() => setLoading(false), 6000)` และเคลียร์เมื่อ finally |

---

## 11. ชื่อแอปบน launcher ไม่เปลี่ยนตาม productName

| | |
|---|---|
| สาเหตุ | ชื่อที่แสดงบนมือถืออ่านจาก `res/values/strings.xml` ไม่ใช่ `tauri.conf.json` |
| ไฟล์ที่แก้ | `app/src/main/res/values/strings.xml` → `app_name`, `main_activity_title` |

---

## 12. APK บวม / แพ็กไฟล์เก่าซ้ำ

| | |
|---|---|
| อาการ | APK 47 MB (เนื้อหาจริง 27 MB) หรือมี `index-*.js` หลายไฟล์ |
| สาเหตุ 1 | คัดลอก `.so` ทับระหว่างที่ Gradle กำลังแพ็ก → ข้อมูลค้าง ~20 MB |
| สาเหตุ 2 | โฟลเดอร์ `assets` สะสมไฟล์ frontend ของทุก build |
| วิธีแก้ | ลบ `app/build/outputs/apk` และ `app/src/main/assets` **ก่อน** ทุกครั้ง (ทำในสคริปต์แล้ว) |

---

## 13. `npx tauri android build` ใช้ไม่ได้บนเครื่องนี้

| | |
|---|---|
| อาการ 1 | `Failed to create a symbolic link ... Creation symbolic link is not allowed` (Windows ไม่ได้เปิด Developer Mode) |
| อาการ 2 | Gradle task `rustBuild*` เรียก `npm.bat` ไม่ได้ (Java spawn ไม่ได้บน Windows) |
| อาการ 3 | ทุกครั้งที่รัน `tauri android build` ไฟล์ Kotlin ที่แก้ไว้จะถูก **เขียนทับ** |
| วิธีแก้ | ใช้ `scripts/build-apk-local.cmd` แทน (ดูข้อ 14) |

---

## 14. สคริปต์ build ที่ใช้จริง

```
scripts/build-apk-local.cmd
```

ลำดับงาน:
1. `vcvars64.bat` + ตั้ง `JAVA_HOME`, `ANDROID_HOME`, `NDK_HOME`
2. `CARGO_HOME=<โปรเจกต์>\.cargo`, `CARGO_TARGET_DIR=D:\alux-target` (path ห้ามมีช่องว่าง เพราะ NDK linker wrapper พัง)
3. ตั้ง linker: `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`, `CC_`, `CXX_`, `AR_`, `RANLIB_`
4. `npm run build`
5. `cargo build --package alux --target aarch64-linux-android --features tauri/custom-protocol --lib`
6. คัดลอก `.so` → `jniLibs/arm64-v8a/`, ล้าง+คัดลอก `dist` → `assets/`
7. ลบ `app/build/outputs/apk` แล้ว `gradlew assembleDebug` (ข้าม task `rustBuild*`)

หลัง patch ไฟล์ Kotlin ให้รัน:
```
pwsh -File scripts/patch-android-webview.ps1
```
(สคริปต์คัดลอกเทมเพลตจาก `scripts/android/` กลับไป ครอบคลุม `RustWebViewClient.kt`, `RustWebView.kt`, `MainActivity.kt`)

---

## 15. การแจกไฟล์ให้เครื่องจริง

| วิธี | ใช้ได้ไหม | หมายเหตุ |
|---|---|---|
| HTTP ในวง LAN | ✅ | `python scripts/serve-apk.py` พอร์ต 8765 — รองรับ Range, เปิด `/` แล้ว 302 ไปไฟล์ |
| MTP (โอนไฟล์ผ่าน USB) | ✅ แต่ช้า | Windows รายงานขนาดเป็น 0 ไบต์เสมอ → **ต้องอ่านไฟล์กลับมาเทียบ SHA-256** |
| ADB | ยังไม่ได้ | ต้องเปิด USB debugging (ยังไม่ได้เปิด) |
| ลบไฟล์ผ่าน MTP | ❌ | `InvokeVerb('delete')` ค้าง |

---

## สรุปไฟล์ที่ถูกแก้ (เทียบกับโปรเจกต์ที่ Tauri generate)

```
src-tauri/src/lib.rs                                  ← mobile_controls_js, clear_browsing_data,
                                                        apps_install_demo, close protocol, parse_url
src-tauri/tauri.conf.json                             ← productName "IVEC SR 2"
src-tauri/assets/demo-app/index.html                  ← แอปตัวอย่างที่ฝังในไบนารี
src/App.tsx                                           ← watchdog กันสปินเนอร์ค้าง, installDemo
src/components/NewTabPage.tsx                         ← ซ่อนทางลัดบน Android, ปุ่มล้างคุกกี้/แอปตัวอย่าง
src/lib/bridge.ts                                     ← clearBrowsingData, appsInstallDemo
src-tauri/gen/android/.../generated/RustWebViewClient.kt ← intent:// + scheme ภายนอก
src-tauri/gen/android/.../generated/RustWebView.kt     ← คุกกี้บุคคลที่สาม + ล้างคุกกี้จริง
src-tauri/gen/android/.../MainActivity.kt              ← back navigation + window insets
src-tauri/gen/android/.../res/values/strings.xml       ← ชื่อแอป
```

> ไฟล์ใน `src-tauri/gen/android/**/generated/` ถูกเขียนใหม่ทุกครั้งที่รัน `tauri android build`
> — ใช้ `scripts/patch-android-webview.ps1` เพื่อใส่กลับ
