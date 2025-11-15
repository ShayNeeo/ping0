# ping0 Frontend (React + TypeScript + Vite)

Single-page app for `ping0`, styled with a minimal monochrome folder vibe.

## 1) Local Development

Prereqs: Node.js and npm installed.

```bash
npm install
npm run dev
```

Configure API base URL via Vite ENV:

Create `.env` in `frontend/`:

```
VITE_API_BASE_URL=http://127.0.0.1:8080
```

## 2) Build

```bash
npm run build
# output in dist/
```

## 3) Deploy to Cloudflare Pages

- Project settings:
  - Build command: `npm run build`
  - Output directory: `dist`
- Environment variable: set `VITE_API_BASE_URL` to your backend API URL, e.g., `https://api.w9.se`.

## Notes
- The app calls `POST /api/upload` on the backend with `multipart/form-data` containing:
  - `content`: URL string or File object
  - `qr_required`: "true" or "false"
- Success response expects:
  - `{ success: true, short_url: string, qr_code_data: string | null }`
- Error response expects:
  - `{ success: false, error: string }`
