# IVEC SR 2 — ชุดพอร์ตสำหรับ iOS (แยกจาก Android)

ไฟล์นี้คือ "อีกรายการ" ที่แยกจาก Android: ระบุว่า **อะไรพร้อมแล้ว**, **อะไรต้องทำบน macOS**, และ
**ข้อจำกัดที่แก้ไม่ได้จากฝั่ง Windows**

> ⚠️ เครื่องนี้เป็น Windows — **คอมไพล์เป็นไฟล์ `.ipa` ไม่ได้** เพราะ iOS ต้องใช้ Xcode + macOS
> เอกสารนี้จึงเป็นชุดตั้งค่า/แผนงานที่พร้อมใช้ทันทีเมื่อเปิดเครื่อง Mac

---

## 1. สิ่งที่ใช้ร่วมกับ Android ได้เลย (ไม่ต้องแก้)

| เรื่อง | สถานะ | หมายเหตุ |
|---|---|---|
| โค้ดหน้าจอหลังบ้าน (React) | ✅ | ใช้ชุดเดียวกัน — **ช่องค้นหา Google แสดงทุกแพลตฟอร์ม** |
| ซ่อนการ์ดทางลัดบนมือถือ | ✅ | `NewTabPage.tsx` ตรวจ `Android\|iPhone\|iPad\|iPod` |
| แถบเครื่องมือล่างของหน้าเว็บ (← → ⟳ ⌂) | ✅ | `#[cfg(mobile)] mobile_controls_js()` ทำงานทั้ง Android และ iOS |
| การสร้างแท็บ = หน้าต่างแยก | ✅ | `#[cfg(mobile)] tab_open()` ใช้ `WebviewWindowBuilder` |
| แอป Alux Apps + แอปตัวอย่าง | ✅ | ใช้ App Data Directory ของแต่ละแพลตฟอร์ม |
| การแปลง URL (IP ในวง LAN → http) | ✅ | `parse_url()` ไม่ผูกกับแพลตฟอร์ม |
| UA ของ Chrome/Safari | ✅ | `chrome_ua()` คืน Safari UA บน iOS |

## 2. สิ่งที่ต้องต่างจาก Android (แก้ไว้แล้วในโค้ด)

| เรื่อง | Android | iOS |
|---|---|---|
| ที่อยู่ของ custom protocol | `http://alux.localhost/close` | `alux://localhost/close` |
| หน้าหลักของแอป | `http://tauri.localhost/` | `tauri://localhost/` |
| ไฟล์ตั้งค่าเฉพาะ | — | `src-tauri/tauri.ios.conf.json` |

`mobile_controls_js()` เลือก URL ให้เองตาม user agent:

```js
var isIOS = /iPhone|iPad|iPod/i.test(navigator.userAgent);
var CLOSE_URL = isIOS ? 'alux://localhost/close' : 'http://alux.localhost/close';
var HOME_URL  = isIOS ? 'tauri://localhost/'   : 'http://tauri.localhost/';
```

## 3. สิ่งที่ต้องทำบนเครื่อง Mac (ทำครั้งเดียว)

```bash
# 1) เครื่องมือ
xcode-select --install                      # Xcode Command Line Tools
# ติดตั้ง Xcode จาก App Store แล้วเปิดครั้งแรกเพื่อยอมรับสัญญาอนุญาต

# 2) Rust target
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# 3) ในโฟลเดอร์โปรเจกต์
npm install
npx tauri ios init          # สร้างโฟลเดอร์ src-tauri/gen/apple
npx tauri ios dev           # รันบน Simulator
npx tauri ios build         # สร้างไฟล์ .ipa
```

## 4. ข้อจำกัดของ iOS ที่ต้องรู้ก่อนแจก

| เรื่อง | รายละเอียด |
|---|---|
| ติดตั้งบนเครื่องคนอื่น | ต้องมี **Apple Developer Program** (ปีละ ~3,000 บาท) และต้องเซ็นชื่อแอป — ไม่มีปุ่ม "ติดตั้ง APK" แบบ Android |
| ฟรีแอคเคานต์ (ไม่จ่าย) | ติดตั้งได้เฉพาะเครื่องตัวเอง/Simulator และ **แอปหมดอายุใน 7 วัน** ต้อง build ใหม่ |
| App Store | แอปที่เป็นเบราว์เซอร์ต้องใช้ WebKit (WKWebView) เท่านั้น — IVEC SR 2 ใช้ WebKit อยู่แล้วจึงผ่านกติกา |
| ล็อกอิน Google | WKWebView ใช้ Safari UA ผ่านได้ แต่ Google อาจเตือน "เบราว์เซอร์ไม่ปลอดภัย" เป็นบางครั้ง (เหมือน Android ก่อนแก้) |
| การแจกผ่านเว็บ | ทำได้เฉพาะ **Ad Hoc / TestFlight** ซึ่งต้องมีบัญชีนักพัฒนาอยู่ดี |

## 5. ตัวแปรที่ต้องตรวจบน iOS ( checklist )

- [ ] หน้าแรกแสดง **ช่องค้นหา** และ **ไม่แสดงการ์ดทางลัด** (เหมือน Android)
- [ ] กดการ์ดแอป → เปิดหน้าต่างใหม่ พร้อมแถบล่าง ← → ⟳ ⌂
- [ ] ปุ่ม ⌂ กลับหน้าหลักได้ (ใช้ `alux://localhost/close`)
- [ ] ลิงก์ `intent://`/`market://` เปิดแอปในเครื่อง — บน iOS ใช้ `tel:`, `mailto:` และ Universal Links แทน (ไม่มี `intent://`)
- [ ] แอปตัวอย่างติดตั้งและเปิดได้ (ใช้ App Data Directory)
- [ ] ล็อกอิน Google Docs ผ่าน

## 6. ความต่างของโค้ดที่อาจต้องเพิ่มบน iOS

1. **Universal Links / `target="_blank"`** — iOS เปิดลิงก์ภายนอกด้วย `openURL` ต้องดักใน `WKNavigationDelegate`
   (wry จัดการให้บางส่วน แต่ควรทดสอบ)
2. **Safe area** — iPhone มีรอยบาก: แถบล่างใช้ `env(safe-area-inset-bottom)` อยู่แล้ว ควรตรวจบนเครื่องจริง
3. **ไม่มี `usesCleartextTraffic`** — iOS ต้องเพิ่ม `NSAppTransportSecurity` ถ้าจะเปิดเว็บ `http://` ในวง LAN
   (เช่น หน้า `192.168.1.39:8765`) ใน `Info.plist`:

```xml
<key>NSAppTransportSecurity</key>
<dict>
  <key>NSAllowsArbitraryLoads</key><true/>
</dict>
```

## 7. สรุปสถานะ

| แพลตฟอร์ม | สถานะ | ไฟล์ที่ส่งมอบ |
|---|---|---|
| Windows | ✅ พร้อมแจก | `release/IVEC SR 2.exe`, `IVEC SR 2_0.1.0_x64-setup.exe` |
| Android | ✅ พร้อมแจก + ทดสอบบนเครื่องจริง | `release/IVEC-SR2-0.1.0-arm64-debug.apk` |
| iOS | 🟡 พร้อมพอร์ต — ต้อง build บน macOS | `src-tauri/tauri.ios.conf.json` + เอกสารนี้ |

> ขั้นต่อไปบน Mac: รัน `npx tauri ios init` แล้ว `npx tauri ios build`
> ถ้าเจอ error ให้เก็บ log ส่งมา — โครงสร้างฝั่ง Rust/หน้าจอพร้อมอยู่แล้ว
