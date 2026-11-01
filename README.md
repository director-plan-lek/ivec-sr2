# Alux (อาเล็ก)

เบราว์เซอร์น้ำหนักเบา เขียนด้วย **TypeScript + Rust (Tauri v2)** ทำงานบน Windows, Linux,
Android และ iOS — เปิด Google Docs ได้ และมี **ช่องติดตั้งแอปในเบราว์เซอร์** (Alux Apps)
สำหรับแอปที่คุณพัฒนาเอง

---

## สถาปัตยกรรมนี้เรียกว่าอะไร

Alux เป็น **WebView-based browser shell** (เปลือกเบราว์เซอร์ที่ใช้ engine ของระบบ)
ไม่ใช่การเขียน browser engine ใหม่ และไม่ใช่ Chromium fork

| แนวทาง | ตัวอย่างจริง | ขนาด | ความเข้ากันได้กับเว็บ |
| --- | --- | --- | --- |
| **WebView shell** ← Alux | Tauri, WebView2, WKWebView | ~5–15 MB | สูง (engine ของ OS) |
| Chromium fork | Brave, Edge, Opera | 150–400 MB | สูงสุด |
| เขียน engine เอง | — | ทำไม่ได้จริงสำหรับคนเดียว | — |

engine ที่ Alux ใช้ต่อแพลตฟอร์ม:

| แพลตฟอร์ม | Engine | หมายเหตุ |
| --- | --- | --- |
| Windows | WebView2 (Chromium) | มากับ Windows 10/11 |
| Linux | WebKitGTK | ต้องติดตั้ง `webkit2gtk-4.1` |
| Android | Android System WebView (Chromium) | มากับเครื่อง |
| iOS | WKWebView (WebKit) | Apple บังคับให้ใช้ WebKit เท่านั้น |

## ฟีเจอร์ที่มีแล้ว

- แท็บจริง (แต่ละแท็บ = child webview แยกกัน ไม่ใช่ `<iframe>` → Google Docs ไม่โดนบล็อก)
- ช่องที่อยู่ + ค้นหา Google, ปุ่มย้อนกลับ/ไปข้างหน้า/โหลดใหม่, คีย์ลัด `Ctrl+T/W/L/R`
- หน้าแท็บใหม่พร้อมทางลัด (Docs, Drive, Gmail, YouTube, Sheets, Wikipedia)
- **Alux Apps**: ติดตั้งแอปจากโฟลเดอร์, เปิดในแท็บ, ถอนการติดตั้ง, เก็บไว้ในเครื่อง
- เสิร์ฟแอปผ่าน protocol ส่วนตัว `alux://` (sandbox ต่อแอป, ไม่มีสิทธิ์เรียกคำสั่งของเบราว์เซอร์)

## เริ่มใช้งาน (โหมดพัฒนา)

```powershell
npm install
npx tauri dev
```

## บิลด์เป็นไฟล์แจกจ่าย

```powershell
npx tauri build
```

ผลลัพธ์อยู่ที่ `src-tauri/target/release/` และตัวติดตั้งอยู่ที่
`src-tauri/target/release/bundle/nsis/`

ดูรายละเอียดทุกแพลตฟอร์มได้ที่ [docs/BUILD.md](docs/BUILD.md) และรูปแบบแอปที่
[docs/ALUX-APPS.md](docs/ALUX-APPS.md)

## โครงสร้างโปรเจกต์

```
alux/
├─ src/                      # เบราว์เซอร์ UI (React + TypeScript)
│  ├─ App.tsx                # แท็บ, แถบที่อยู่, การ sync webview
│  ├─ lib/bridge.ts          # ตัวเรียกคำสั่ง Rust (invoke)
│  └─ components/            # NewTabPage, AppsPanel, Icons
├─ src-tauri/
│  ├─ src/lib.rs             # แท็บ/แอป/protocol alux:// ทั้งหมด
│  ├─ tauri.conf.json        # ชื่อแอป, ไอคอน, ตัวติดตั้ง
│  └─ capabilities/          # สิทธิ์ของ webview หลัก
├─ apps/hello-alux/          # แอปตัวอย่าง (ทดสอบช่องติดตั้งแอป)
└─ docs/
```
