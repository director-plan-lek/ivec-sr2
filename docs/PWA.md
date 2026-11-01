# IVEC SR 2 — PWA (ติดตั้งบน iPhone ได้ฟรี ไม่ต้องมี Mac)

โฟลเดอร์ [`pwa/`](../pwa) คือเวอร์ชันเว็บที่ใช้ **"เพิ่มไปยังหน้าจอโฮม"** ของ Safari/Chrome
ได้ผลเหมือนแอปติดตั้ง (มีไอคอน, เปิดเต็มจอ) — **ไม่มีค่าใช้จ่าย ไม่ต้องมี Apple Developer ไม่ต้องมี Mac**

## สิ่งที่ได้

| | |
|---|---|
| หน้าแรก | ชื่อ IVEC SR 2 + ช่องค้นหา Google + การ์ดทางลัด 6 รายการ |
| ติดตั้ง | iOS: Share ⤴ → "เพิ่มไปยังหน้าจอโฮม" · Android: เมนู ⋮ → "ติดตั้งแอป" |
| หน้าตา | เปิดเต็มจอแบบแอป (standalone) ไม่มีแถบที่อยู่ของเบราว์เซอร์ |
| ออฟไลน์ | service worker แคชหน้าแรกไว้ เปิดได้แม้ไม่มีเน็ต (ตัวเว็บอื่นต้องมีเน็ตตามปกติ) |
| ค่าใช้จ่าย | **ฟรี** (GitHub Pages) |

## ข้อจำกัดที่ต้องเข้าใจ

> PWA **ไม่ใช่เบราว์เซอร์** — Apple ไม่อนุญาตให้แอปที่ไม่ใช่ Safari ใช้ engine อื่น
> ฉะนั้นกดการ์ดแล้วจะเปิดใน Safari (หรือแท็บใหม่) ไม่ใช่ในหน้าต่างของ IVEC SR 2
> **ถ้าต้องการเบราว์เซอร์เต็มรูปแบบบน iPhone ต้องใช้แอปเนทีฟ + Apple Developer ($99/ปี)**

## วิธี deploy (ครั้งเดียว ประมาณ 3 นาที)

1. push โค้ดขึ้น GitHub (ต้องมี repo ก่อน)
2. ไปที่ repo → **Settings → Pages**
3. หัวข้อ **Build and deployment → Source** เลือก **GitHub Actions**
4. ไปที่แท็บ **Actions** → run **`deploy-pwa`** จะทำงานเอง (หรือกด Run workflow)
5. รอ ~1 นาที → เปิด URL ที่ได้:
   ```
   https://<username>.github.io/ivec-sr2/
   ```

> workflow [`pages.yml`](../.github/workflows/pages.yml) จะ build + deploy ทุกครั้งที่แก้ไฟล์ใน `pwa/`

## ทดสอบบนเครื่องก่อน deploy

```powershell
cd "D:\app\Web Browser\alux\pwa"
python -m http.server 8777
```
เปิด `http://localhost:8777` — ตรวจว่า:
- ช่องค้นหาทำงาน (พิมพ์ `google.com` → เปิดเว็บ, พิมพ์ `สวัสดี` → ค้น Google)
- การ์ดทั้ง 6 เปิดได้
- DevTools → Application → Manifest ไม่มี error
- DevTools → Application → Service Workers ขึ้น `activated`

## วิธีติดตั้งบน iPhone

1. เปิด URL ของเว็บใน **Safari** (ต้อง Safari เท่านั้น — Chrome บน iOS ทำไม่ได้)
2. แตะปุ่ม **Share ⤴** ด้านล่าง
3. เลือก **"เพิ่มไปยังหน้าจอโฮม" (Add to Home Screen)**
4. ตั้งชื่อ → **เพิ่ม**
5. เปิดจากไอคอนบนหน้าจอ → จะเปิดเต็มจอ ไม่มีแถบเบราว์เซอร์

## เพิ่มแอปของหน่วยงาน (ไม่บังคับ)

วางไฟล์ `pwa/apps.json` แบบนี้ แล้วส่วน "IVEC Apps" จะแสดงขึ้นเอง:

```json
[
  {
    "name": "ระบบสารบรรณ",
    "path": "https://example.ac.th/sarabun/",
    "accent": "#2f7dff"
  },
  {
    "name": "ปฏิทินการศึกษา",
    "path": "https://example.ac.th/calendar/",
    "accent": "#7a5cff"
  }
]
```

> `path` เป็นได้ทั้ง URL ภายนอก หรือไฟล์ภายใน `pwa/` (เช่น `./apps/sarabun/index.html`)

## ไฟล์ในโฟลเดอร์นี้

```
pwa/
├── index.html              ← หน้าแรก
├── styles.css              ← ธีมเดียวกับแอป (มืด + gradient)
├── app.js                  ← ค้นหา/แปลง URL, โหลด apps.json, ลงทะเบียน SW
├── sw.js                   ← offline shell
├── manifest.webmanifest    ← ชื่อ, ไอคอน, display standalone
└── icons/
    ├── icon-192.png
    ├── icon-512.png
    ├── icon-maskable-512.png
    └── apple-touch-icon.png
```

## ทางเลือกสำรอง

ถ้าไม่ต้องการ GitHub Pages:

| บริการ | หมายเหตุ |
|---|---|
| Netlify / Cloudflare Pages | ลากโฟลเดอร์ `pwa/` วาง ก็ได้ URL ทันที |
| Python `http.server` ในวง LAN | ใช้ทดสอบชั่วคราว (ไม่มี HTTPS → ติดตั้ง PWA ไม่ได้) |
