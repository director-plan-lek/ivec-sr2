# Build อัตโนมัติด้วย GitHub Actions (ได้ `.ipa` เมื่อมี repo)

ไฟล์ workflow: [`.github/workflows/build.yml`](../.github/workflows/build.yml)

เมื่อ push ขึ้น GitHub ระบบจะสร้างของให้ **4 แพลตฟอร์ม** พร้อมกัน:

| Job | Runner | ได้อะไร | ใช้ทำอะไร |
|---|---|---|---|
| `windows` | windows-latest | `alux.exe`, `IVEC SR 2_*_x64-setup.exe` | แจกให้เครื่อง Windows |
| `linux` | ubuntu-22.04 | `.AppImage`, `.deb` | แจกให้ Linux |
| `android` | ubuntu-latest | `.apk` (debug) | แจกให้มือถือ Android |
| `ios` | macos-latest | `.app` หรือ `.ipa` | ติดตั้งบน iPhone/iPad |

ไฟล์ทั้งหมดดาวน์โหลดได้ที่แท็บ **Actions → เลือก run → Artifacts** (เก็บ 90 วัน)

---

## 1. เตรียม repo (ทำครั้งเดียว)

```bash
cd alux
git init
git add .
git commit -m "IVEC SR 2 — first build"

# สร้าง repo ว่างบน GitHub ชื่อ ivec-sr2 แล้ว
git remote add origin https://github.com/<username>/ivec-sr2.git
git branch -M main
git push -u origin main
```

> **สำคัญ:** workflow นี้ถือว่า **โฟลเดอร์ `alux` คือ root ของ repo**
> ถ้าคุณ push ทั้ง `D:\app\Web Browser` (มีโฟลเดอร์ `alux` ซ้อนข้างใน) ให้แก้ `working-directory` ทุกขั้นและ path ของ artifact

หลัง push ครั้งแรก ไปที่แท็บ **Actions** จะเห็น run ทำงานเอง ใช้เวลา ~8–15 นาที (iOS นานสุด)

---

## 2. ผลลัพธ์ของ iOS — 2 กรณี

### กรณี A: ยังไม่มีบัญชี Apple Developer (ไม่ต้องตั้งค่าเพิ่ม)

workflow จะสร้าง **`IVEC-SR2-device.app`** (ยังไม่เซ็นชื่อ)

วิธีติดตั้ง (เลือกอย่างใดอย่างหนึ่ง):

| เครื่องมือ | ต้องมี | อายุแอป |
|---|---|---|
| **Xcode** (Mac) | Mac + สาย USB | 7 วัน แล้ว build ใหม่ |
| **Sideloadly** (Windows/Mac) | Apple ID ฟรี | 7 วัน |
| **Apple Configurator** | Mac | 7 วัน |

ขั้นตอน Sideloadly (บน Windows — ตรงกับเครื่องคุณ):
1. แตก `ivec-sr2-ios.zip` → ได้ไฟล์ `.app`
2. เปิด Sideloadly → ลาก `.app` ลง → ใส่ Apple ID → Start
3. บน iPhone: ตั้งค่า → ทั่วไป → VPN & Device Management → เชื่อถือโปรไฟล์
4. เปิดแอปได้ (ครบ 7 วันให้ทำซ้ำ)

### กรณี B: มี Apple Developer Program (ได้ `.ipa` จริง)

เพิ่ม **Secrets** 5 ตัวที่ repo → Settings → Secrets and variables → Actions → New repository secret:

| Secret | คืออะไร | วิธีได้มา |
|---|---|---|
| `APPLE_CERTIFICATE` | ไฟล์ `.p12` แปลงเป็น base64 | Keychain Access → Export "Apple Distribution: ..." → `base64 -i cert.p12 \| pbcopy` |
| `APPLE_CERTIFICATE_PASSWORD` | รหัสของ `.p12` | ตอน export |
| `APPLE_SIGNING_IDENTITY` | ชื่อ identity | `security find-identity -v -p codesigning` |
| `APPLE_DEVELOPMENT_TEAM` | Team ID 10 ตัว | developer.apple.com → Membership |
| `APPLE_MOBILE_PROVISION` | ไฟล์ `.mobileprovision` base64 | สร้าง Ad Hoc / Development profile → ดาวน์โหลด → `base64 -i profile.mobileprovision \| pbcopy` |

เมื่อมี secrets ครบ workflow จะสร้าง **`IVEC-SR2.ipa`** → ติดตั้งผ่าน **TestFlight / Ad Hoc / Xcode**

---

## 3. สิ่งที่ workflow ทำกับ iOS

```
npx tauri ios init                       # สร้างโปรเจกต์ Xcode ใน src-tauri/gen/apple
PlistBuddy → NSAllowsArbitraryLoads      # ให้เปิดเว็บ http:// ในวง LAN ได้
security import (ถ้ามี secret)           # ติดตั้ง certificate เข้า keychain
npx tauri ios build --target aarch64-apple-ios
  └─ ไม่มี APPLE_SIGNING_IDENTITY → ได้ .app (unsigned)
  └─ มี APPLE_SIGNING_IDENTITY + provisioning → Tauri เซ็นให้ ได้ .ipa
```

Rust target ที่ติดตั้งบน runner: `aarch64-apple-ios`, `aarch64-apple-ios-sim`

---

## 4. ปัญหาที่อาจเจอ

| อาการ | สาเหตุ/ทางแก้ |
|---|---|
| iOS job แดงตั้งแต่ `tauri ios init` | ยังไม่ได้ตั้ง `bundle.iOS.minimumSystemVersion` — มีไฟล์ `src-tauri/tauri.ios.conf.json` แล้ว ตรวจว่า push ไปด้วย |
| `No signing certificate found` | ไม่ได้ตั้ง secrets → ถือว่าปกติ ใช้กรณี A (unsigned) แทน |
| `altool`/`Transporter` error ตอน export | ต้องมี `APPLE_API_KEY` + `APPLE_API_ISSUER` (App Store Connect API key) เพิ่ม |
| artifact ไม่มีไฟล์ | ดู log ขั้น `Collect artifacts` — path ของ `.app` อาจต่างตามชื่อ product |
| Android job แดงที่ patch script | ต้องมี `pwsh` บน ubuntu (มีให้อยู่แล้ว) และไฟล์ `scripts/android/*.kt` ต้อง push ขึ้นไปด้วย |

---

## 5. ทางลัดถ้าไม่อยากใช้ GitHub

รันบนเครื่อง Mac ที่ไหนก็ได้:

```bash
npx tauri ios build --target aarch64-apple-ios
```

ได้ไฟล์ที่ `src-tauri/gen/apple/build/arm64/…app` เหมือนกัน
