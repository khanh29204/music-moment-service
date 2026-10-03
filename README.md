# Music Moment Service

Service tính byte-range cho phép chèn (inject) một đoạn nhạc tùy chỉnh làm `preview_url` cho moment loại nhạc trong client custom Locket. Không lưu/transcode file nhạc — chỉ quy đổi `ms → byte offset` (MP3 64kbps CBR) và stream phần cần phát từ S3.

Tài liệu luồng tổng thể: xem `music-injection-flow.md` (file đi kèm trong repo client).

## Cách hoạt động

File MP3 từ [yt-metadata-service](https://github.com/khanh29204/yt-metadata-service) luôn là **64kbps CBR, 44.100Hz, không ID3, không bit reservoir**, nên có thể tính chính xác byte offset từ mốc thời gian mà không cần decode:

- Frame duration ≈ 26,12245 ms (`1152 / 44100`), average frame size ≈ 208,97959 bytes.
- `start` dùng `floor`, `end` dùng `ceil`, ép `endFrame ≥ startFrame + 1` — đoạn cắt không hụt cuối, không rỗng.
- Cộng thêm 1 frame (Xing/Info frame ffmpeg ghi ở đầu file) để player đọc đúng duration.
- Trả `actualStartMs/actualEndMs` (mốc đã snap theo frame) để client đồng bộ UI preview.

## API

Cả endpoint `moments` yêu cầu header `x-api-key`. Nếu không cấu hình `API_KEY` trong env, auth bị bỏ qua (dev only).

### `GET /songs/stream?url=<s3Url>&start=<ms>&end=<ms>`

Public (không cần key). Stream đoạn nhạc `[start, end)` từ S3 dưới dạng `audio/mpeg` với header `Range` đã frame-align.

- `start`/`end` là ms, `end > start` bắt buộc.
- Lỗi: `400` thiếu/invalid tham số, `502` upstream lỗi.

### `POST /moments`

Lưu mapping `momentId → inject link` (upsert, trả `201` nếu mới tạo, `200` nếu cập nhật).

```json
{ "momentId": "abc123", "url": "https://.../songs/stream?..." }
```

### `GET /moments/:moment_id`

Trả `{ "url": "<inject link>" }` cho client ghi đè `preview_url`. Không có → `404`.

## Chạy

Cần env (file `.env` hoặc biến môi trường):

```env
PORT=8080
MONGODB_URI=mongodb+srv://...
MONGODB_DB=locket
REDIS_URL=redis://...
API_KEY=...
CORS_ORIGINS=https://example.com,https://app.example.com
```

`CORS_ORIGINS` là danh sách origin phân tách bởi dấu phẩy. Không set → cho mọi origin.

Local:

```sh
cargo run
```

Docker Compose (pull image):

```sh
docker compose up -d
```

MongoDB/Redis nằm ngoài service này (Atlas / instance riêng) — compose chỉ chạy app.
