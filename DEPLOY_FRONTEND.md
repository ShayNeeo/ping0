# Deploying the static frontend (Cloudflare Pages)

This document shows two simple ways to publish the `static/` folder to Cloudflare Pages and how to provide a runtime `config.json` so the frontend knows which backend to use.

Options
- Manual upload (Cloudflare Pages — drag & drop)
- Wrangler CLI + GitHub (or Git integration)

Runtime config
- The frontend reads `/config.json` at startup. Provide a `config.json` file at the root of the published site with the shape:

```json
{
  "apiBase": "https://api.example.com"
}
```

If `apiBase` is omitted, the site will call relative paths (e.g., `/upload`) — useful when serving both frontend and backend from the same origin.

Manual publish (quick)
1. Build (not required for purely static repo): ensure `static/` contains `index.html` and `config.json`.
2. Zip (optional): `cd static && zip -r ../static.zip .`
3. Cloudflare Pages dashboard: Create a new project and choose "Deploy a site without Git" (drag & drop) or use the drag-and-drop upload area to upload the files from `static/`.
4. Test the site and make sure `/config.json` is present and returns your `apiBase`.

Wrangler / Git (recommended for repeatable deploys)
1. Create a Git repo for the frontend (or use the existing one). Commit the `static/` folder to the branch used by Pages.
2. In Cloudflare Pages, connect the repo and set the build settings:
   - Build command: (none) — Pages will simply serve the folder if you point it to `static/` as the build output.
   - Build output directory: `static`
3. In your repo, add `static/config.json` with the production `apiBase` value (or use Cloudflare Pages' environment/variables feature and a small build step to emit `config.json`).

Setting config.json at deploy time (no Git)
- Cloudflare Pages UI doesn't have a way to edit files at runtime; the simplest is to include `config.json` in the uploaded files.
- Alternatively, add a tiny build step (e.g., `echo '{"apiBase":"https://api.example.com"}' > static/config.json`) in your CI or local machine before upload.

CORS considerations
- If the frontend origin (pages) and backend origin (VPS) differ, you'll need to enable CORS on the backend. The server already accepts default requests but double-check `Access-Control-Allow-Origin` if you see CORS errors.
- For production, set `Access-Control-Allow-Origin` to the specific Cloudflare Pages domain (or use a reverse-proxy on the same origin to avoid CORS entirely).

Testing from Pages
1. Deploy the site.
2. Visit the published URL and open DevTools → Network to ensure requests to `/upload` or the configured `apiBase` succeed.

Rollback
- Keep a copy of the previous `config.json` and site files so you can re-upload if needed.

Security note
- Avoid including secrets in `config.json`. It is publicly served.
