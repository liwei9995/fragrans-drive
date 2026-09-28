import { createSHA256 } from 'hash-wasm'

export const calculateFileHash = async (
  file: File,
  onProgress?: (progress: number) => void,
): Promise<string> => {
  try {
    const hasher = await createSHA256()
    hasher.init()

    const chunkSize = 10 * 1024 * 1024 // 10MB chunks
    const totalChunks = Math.ceil(file.size / chunkSize)

    for (let i = 0; i < totalChunks; i++) {
      const start = i * chunkSize
      const end = Math.min(start + chunkSize, file.size)
      const blob = file.slice(start, end)

      // Read the chunk as ArrayBuffer
      const buffer = await blob.arrayBuffer()
      hasher.update(new Uint8Array(buffer))

      if (onProgress) {
        onProgress(Math.round(((i + 1) / totalChunks) * 100))
      }
    }

    return hasher.digest('hex')
  } catch (wasmError) {
    console.warn(
      'WASM SHA256 calculation failed, falling back to Web Crypto API:',
      wasmError,
    )
    if (typeof crypto !== 'undefined' && crypto.subtle) {
      const buffer = await file.arrayBuffer()
      const digest = await crypto.subtle.digest('SHA-256', buffer)
      const hashArray = Array.from(new Uint8Array(digest))
      return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
    }
    throw wasmError
  }
}
