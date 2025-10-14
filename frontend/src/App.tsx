import { useState, useRef, FormEvent } from 'react'

const API_BASE: string = import.meta.env?.VITE_API_BASE_URL || ''

type SuccessResult = {
  success: true
  short_url: string
  qr_code_data: string | null
}

type ErrorResult = {
  success: false
  error: string
}

type Result = SuccessResult | ErrorResult

export default function App() {
  const [urlInput, setUrlInput] = useState('')
  const [fileInput, setFileInput] = useState<File | null>(null)
  const [generateQr, setGenerateQr] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [result, setResult] = useState<SuccessResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [dragOver, setDragOver] = useState(false)
  const [imagePreview, setImagePreview] = useState<string | null>(null)

  const fileRef = useRef<HTMLInputElement | null>(null)

  function handleUrlChange(v: string) {
    setUrlInput(v)
    if (fileInput) {
      // Clear file if URL is being typed
      if (fileRef.current) fileRef.current.value = ''
      setFileInput(null)
    }
  }

  function handleFileChange(file: File | null) {
    setFileInput(file)
    if (file) {
      // Clear URL if a file is chosen
      setUrlInput('')
      if (file.type.startsWith('image/')) {
        const url = URL.createObjectURL(file)
        setImagePreview(url)
      } else {
        setImagePreview(null)
      }
    }
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault()
    setIsLoading(true)
    setError(null)
    setResult(null)

    try {
      // Validate inputs
      const hasUrl = urlInput.trim().length > 0
      const hasFile = !!fileInput
      if (!hasUrl && !hasFile) {
        setError('Please provide a URL or choose a file.')
        return
      }
      if (hasUrl && hasFile) {
        setError('Provide only one: URL or File, not both.')
        return
      }

      const form = new FormData()
      form.set('qr_required', generateQr ? 'true' : 'false')

      if (hasUrl) {
        form.set('content', urlInput.trim())
      } else if (hasFile && fileInput) {
        form.set('content', fileInput)
      }

      const resp = await fetch(joinUrl(API_BASE, '/api/upload'), {
        method: 'POST',
        body: form,
      })

      const data = (await resp.json()) as Result
      if (!resp.ok || (data as ErrorResult).success === false) {
        const msg = (data as ErrorResult).error || `HTTP ${resp.status}`
        throw new Error(msg)
      }

      const ok = data as SuccessResult
      setResult(ok)
    } catch (err: any) {
      setError(err?.message || 'Unexpected error')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="app">
      <main className="container">
        <h1>ping0</h1>
        <p className="subtitle">Share a link or upload a file · get a short URL</p>

        <form onSubmit={handleSubmit} className="form">
          <div
            className={`dropzone ${dragOver ? 'dragover' : ''}`}
            onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
            onDragLeave={() => setDragOver(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragOver(false);
              const f = e.dataTransfer.files?.[0]
              if (f) handleFileChange(f)
            }}
            onPaste={(e) => {
              const items = e.clipboardData?.items
              if (!items) return
              for (let i = 0; i < items.length; i++) {
                const it = items[i]
                if (it.kind === 'file') {
                  const f = it.getAsFile()
                  if (f) { handleFileChange(f); break }
                }
                if (it.kind === 'string') {
                  it.getAsString((text) => {
                    if (/^https?:\/\//i.test(text.trim())) handleUrlChange(text.trim())
                  })
                }
              }
            }}
          >
            <div>Drop a file here, or paste a file or URL</div>
          </div>

          <label className="label">
            URL
            <input
              type="text"
              placeholder="https://example.com"
              value={urlInput}
              onChange={(e) => handleUrlChange(e.target.value)}
              className="input"
            />
          </label>

          <div className="or">or</div>

          <label className="label">
            File
            <input
              ref={fileRef}
              type="file"
              onChange={(e) => handleFileChange(e.target.files?.[0] || null)}
              className="input"
            />
          </label>

          {imagePreview && (
            <div className="preview">
              <img src={imagePreview} alt="preview" />
            </div>
          )}

          <label className="checkbox">
            <input
              type="checkbox"
              checked={generateQr}
              onChange={(e) => setGenerateQr(e.target.checked)}
            />
            Generate QR Code
          </label>

          <button type="submit" className="button" disabled={isLoading}>
            {isLoading ? 'Submitting…' : 'Create'}
          </button>
        </form>

        {isLoading && <div className="status">Submitting…</div>}
        {error && <div className="error">{error}</div>}
        {result && (
          <div className="result">
            <div className="row">
              <span className="label-inline">Short URL</span>
              <a href={toAbsoluteUrl(result.short_url)} className="link" target="_blank" rel="noreferrer">
                {toAbsoluteUrl(result.short_url)}
              </a>
            </div>
            {result.qr_code_data && (
              <div className="qr">
                <img src={result.qr_code_data} alt="QR code" />
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  )
}

function joinUrl(base: string, path: string) {
  if (!base) return path
  if (!path.startsWith('/')) path = '/' + path
  return base.replace(/\/+$/, '') + path
}

function toAbsoluteUrl(u: string) {
  try {
    return new URL(u, window.location.origin).href
  } catch {
    return u
  }
}
