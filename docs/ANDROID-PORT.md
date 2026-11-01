# Android — สถานะการ build (.apk)

**สถานะ: build .apk ได้แล้ว** — ไฟล์ `release/Alux-0.1.0-arm64-debug.apk` (26.3 MB, debug-signed, ABI arm64-v8a)
ยังไม่ทดสอบบนมือถือ/อีมูเลเตอร์จริง (เครื่องนี้ไม่มี adb device)

## ทำไมถึงต้องมีขั้นตอนพิเศษ

Tauri ออกแบบให้ `tauri android build` จัดการทุกอย่าง แต่บนเครื่อง Windows ที่ path มีช่องว่าง
(`D:\app\Web Browser`) และไม่มีสิทธิ์สร้าง symlink มี 4 จุดที่ต้องแก้:

| ปัญหา | อาการ | วิธีแก้ที่ใช้ |
|---|---|---|
| MSVC ไม่ถูก init | `cl.exe D8050 cannot execute c1xx.dll` | เรียก `vcvars64.bat` ก่อน build |
| `-Z7` debug record | `failed to get command line into debug records` | `CARGO_PROFILE_DEV_DEBUG=0` |
| plugin เขียนใน `~/.cargo` ไม่ได้ | `failed to create .tauri directory: Access denied` | `CARGO_HOME=<workspace>\.cargo` |
| NDK linker เป็น `.cmd` | `linker-arguments"" was unexpected at this time` | `CARGO_TARGET_DIR=D:\alux-target` (path ไม่มีช่องว่าง) |
| ไม่มีสิทธิ์ symlink | `Creation symbolic link is not allowed` ตอน copy `.so` → `jniLibs` | คัดลอก `.so` + assets เอง แล้วรัน Gradle |
| Gradle เรียก `npm` ไม่ได้ | `A problem occurred starting process 'command 'npm.bat''` | `gradlew assembleDebug -x rustBuild*` |

## ขั้นตอน build จริงที่ใช้ได้บนเครื่องนี้

```powershell
# 1) Rust + assets ขึ้นไปถึงขั้น link .so (สคริปต์จัดการ env ทั้งหมดให้)
cmd /c "D:\app\Web Browser\alux\scripts\build-android.cmd"
#    ผลลัพธ์: D:\alux-target\aarch64-linux-android\debug\libalux_lib.so

# 2) คัดลอกสิ่งที่ tauri CLI ทำไม่สำเร็จ (เพราะไม่มีสิทธิ์ symlink)
$gen = "D:\app\Web Browser\alux\src-tauri\gen\android"
Copy-Item "D:\app\Web Browser\alux\dist\*" "$gen\app\src\main\assets" -Recurse -Force
New-Item -ItemType Directory -Force "$gen\app\src\main\jniLibs\arm64-v8a" | Out-Null
Copy-Item "D:\alux-target\aarch64-linux-android\debug\libalux_lib.so" "$gen\app\src\main\jniLibs\arm64-v8a\"

# 3) ประกอบ APK ด้วย Gradle (ข้ามงาน Rust ที่เรียก npm ไม่ได้)
cd $gen
.\gradlew.bat assembleDebug --no-daemon -x rustBuildArmDebug -x rustBuildArm64Debug -x rustBuildX86Debug -x rustBuildX86_64Debug
#    ผลลัพธ์: app\build\outputs\apk\arm64\debug\app-arm64-debug.apk
```

## ถ้าอยากได้ build ที่สะอาด (ไม่ต้องใช้ท่าข้างบน)

ใช้ CI ที่เตรียมไว้: `.github/workflows/build.yml` job `android`
รันบน Ubuntu มี JDK 17 + SDK + NDK ครบ และไม่มีปัญหา path/symlink แบบ Windows
สั่ง `npx tauri android init && npx tauri android build --apk` ได้ตรง ๆ

## สถาปัตยกรรมแท็บบนมือถือ (แก้แล้ว แต่ยังไม่ทดสอบจริง)

`Window::add_child` เป็น desktop-only:

```rust
#[cfg(any(test, all(desktop, feature = "unstable")))]
pub fn add_child<P: Into<Position>, S: Into<Size>>(...)
```

ใน `src-tauri/src/lib.rs` จึงมีคำสั่ง 2 ชุด:

- `#[cfg(desktop)]` — `tab_open`/`tab_sync`/`tab_close`/`app_open` ใช้ child webview ซ้อนในหน้าต่างเดียว
- `#[cfg(mobile)]` — หนึ่งแท็บ = หนึ่ง `WebviewWindow` (`WebviewWindowBuilder::new(...).build()`),
  `tab_sync` เป็น no-op, `tab_close` ปิดหน้าต่าง

ผลที่ตามมา: UX มือถือยังเป็น UI เดสก์ท็อป (tab strip แนวนอน, ปุ่ม back ของ Android ยังไม่ผูกกับ `tab_action("back")`)

## งานที่เหลือก่อนปล่อยให้อาเล็กใช้จริง

1. ทดสอบบนมือถือจริง (`adb install app-arm64-debug.apk`) — ยังไม่ได้ทำ
2. ผูกปุ่ม back ของ Android เข้ากับการย้อนกลับในแท็บ
3. Build release + keystore ของตัวเอง (ตอนนี้ debug-signed, ใช้แจกไม่ได้จริง)
4. ลดขนาด: build `--release --target aarch64` จะได้ไฟล์เล็กกว่านี้มาก (ตอนนี้ 26 MB เพราะ debug)
