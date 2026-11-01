# ติดตั้งแอปบน iPhone — 3 ทางที่ทำได้จริง (จากเครื่อง Windows)

> สรุปสั้น: **Apple Developer Program ไม่ใช่โปรแกรมที่ติดตั้งในเครื่อง** — เป็นบัญชีสมาชิกรายปี
> (99 USD ≈ 3,000 บาท) ที่ให้สิทธิ์สร้าง certificate + provisioning profile
> ส่วน **การคอมไพล์ `.ipa` ยังต้องใช้ macOS/Xcode เสมอ** ไม่ว่าจะมีบัญชีหรือไม่

---

## ทางที่ 1 — Apple ID ฟรี + Sideloadly (ทำได้วันนี้ บน Windows)

| | |
|---|---|
| ค่าใช้จ่าย | ฟรี |
| ต้องมี | Windows, สาย USB, Sideloadly, iTunes/Apple Devices (สำหรับไดรเวอร์) |
| อายุแอป | **7 วัน** แล้วต้องเซ็นใหม่ |
| จำนวนแอป | 3 แอปต่อ Apple ID |

ขั้นตอน:
1. GitHub Actions → โหลด artifact `ivec-sr2-ios` → แตกได้ไฟล์ `IVEC SR 2.app`
2. เปิด **Sideloadly** → ลาก `.app` ลงช่อง → ใส่ Apple ID
3. กด Start → ใส่รหัสผ่านแอป (App-Specific Password ถ้าเปิด 2FA)
4. บน iPhone: ตั้งค่า → ทั่วไป → VPN & Device Management → **เชื่อถือโปรไฟล์**
5. เปิดแอปได้ — ครบ 7 วันทำขั้น 2–3 ซ้ำ

> เหมาะสำหรับทดลองเอง ไม่เหมาะแจกจ่ายเพราะทุกเครื่องต้อง sideload เอง

---

## ทางที่ 2 — Apple Developer Program + GitHub Actions (แจกได้จริง)

| | |
|---|---|
| ค่าใช้จ่าย | 99 USD/ปี |
| ต้องมี | บัญชี Apple Developer (สมัครจากเว็บได้ ไม่ต้องมี Mac) + repo GitHub |
| ได้ไฟล์ | `IVEC-SR2.ipa` ติดตั้งผ่าน **TestFlight**, **Ad Hoc** หรือ Xcode |
| อายุแอป | 1 ปี (ตามอายุ certificate) |

สิ่งที่ต้องเตรียม (ทำบน Windows ได้ทั้งหมด ยกเว้นขั้น export certificate):

1. สมัครที่ developer.apple.com (ยืนยันตัวตน 1–3 วัน)
2. สร้าง **Certificate** ชนิด Distribution (ต้องใช้ Mac/Keychain เพื่อ export `.p12` — หรือใช้ OpenSSL บน Windows ก็ได้)
3. สร้าง **App ID** = `com.alux.browser`
4. สร้าง **Provisioning Profile** (Ad Hoc ถ้าจะแจกตรง หรือ App Store Connect ถ้าจะใช้ TestFlight)
5. ใส่ 5 secrets ใน repo GitHub ตาม `GITHUB-ACTIONS.md`
6. push → รอ ~12 นาที → โหลด `.ipa`
7. ติดตั้ง: เปิด `.ipa` ด้วย Xcode (Mac) / อัปโหลดขึ้น TestFlight / ใช้ Apple Configurator

> **หัวใจสำคัญ:** แม้ไม่มี Mac ในบ้าน GitHub Actions (macos-latest) จะทำหน้าที่เป็น Mac ให้ — นี่คือทางที่ใช้ได้จริงสำหรับคุณ

---

## ทางที่ 3 — เช่า Mac ในคลาวด์ (ถ้าอยากกดปุ่มเอง)

| บริการ | ราคาโดยประมาณ | หมายเหตุ |
|---|---|---|
| MacinCloud | ~1 USD/ชม. | ใช้ Xcode ผ่าน remote desktop |
| MacStadium | ~80 USD/เดือน | เครื่องเต็ม |
| GitHub Actions | ฟรี (public repo) | ไม่มีหน้าจอ ต้องเขียน workflow |

ใช้เมื่อ: ต้องดีบั๊กบน Xcode โดยตรง หรือต้องทดสอบบน Simulator แบบเห็นหน้าจอ

---

## สรุปการตัดสินใจ

| สถานการณ์ | เลือก |
|---|---|
| แค่อยากลองบน iPhone เครื่องตัวเอง | ทางที่ 1 (ฟรี, 7 วัน) |
| ต้องการให้อาเล็ก / ครู / นักศึกษาใช้ด้วย | ทางที่ 2 (99 USD + TestFlight) |
| ต้องแก้บั๊ก UI บน iOS บ่อย ๆ | ทางที่ 3 หรือซื้อ Mac mini มือสอง |

## คำถามที่พบบ่อย

**มีบัญชี $99 แล้ว ติดตั้งบนเครื่อง Windows ได้ไหม?**
ไม่ได้ครับ — บัญชีให้สิทธิ์ *เซ็น* แต่ตัวแอปต้องถูกสร้างบน macOS ก่อน (ผ่าน CI หรือ Mac)

**ไม่มี Mac เลย ทำได้ไหม?**
ได้ — ทางที่ 1 และ 2 ไม่ต้องมี Mac ในบ้าน แต่ทางที่ 2 ต้องใช้ GitHub Actions เป็นตัว build

**ทำไม Android แจกง่ายกว่า?**
Android อนุญาตให้ติดตั้งไฟล์ `.apk` ที่เซ็นเองได้ (sideload) ส่วน Apple บังคับให้ทุกแอปผ่านการเซ็นด้วยบัญชีนักพัฒนา
