import React, { useState } from 'react'
import axios from 'axios'

export default function App() {
  const [file, setFile] = useState<File | null>(null)
  const [result, setResult] = useState<any>(null)

  const upload = async () => {
    if (!file) return
    const fd = new FormData()
    fd.append('file', file)
    const r = await axios.post('http://127.0.0.1:8080/upload', fd, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    setResult(r.data)
  }

  return (
    <div className="container">
      <h1>ping0</h1>
      <input type="file" onChange={e => setFile(e.target.files?.[0] ?? null)} />
      <button onClick={upload}>Upload</button>
      {result && (
        <div>
          <p>Link: <a href={result.link}>{result.link}</a></p>
          <div dangerouslySetInnerHTML={{ __html: result.qr_svg }} />
        </div>
      )}
    </div>
  )
}
