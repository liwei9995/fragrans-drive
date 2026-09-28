import { describe, expect, it, vi } from 'vitest'
import { calculateFileHash } from './fileHash'

const mockCreateSHA256 = vi.fn()

vi.mock('hash-wasm', () => ({
  createSHA256: (...args: unknown[]) => mockCreateSHA256(...args),
}))

describe('calculateFileHash', () => {
  it('calculates sha256 hash correctly for a file using hash-wasm', async () => {
    const mockHasher = {
      init: vi.fn(),
      update: vi.fn(),
      digest: vi
        .fn()
        .mockReturnValue(
          '64ec88ca00b268e5ba1a35678a1b5316d212f4f366b2477232534a8aeca37f3c',
        ),
    }
    mockCreateSHA256.mockResolvedValueOnce(mockHasher)

    const content = 'Hello world'
    const file = new File([content], 'hello.txt', { type: 'text/plain' })
    const progressCalls: number[] = []

    const hash = await calculateFileHash(file, (percent) => {
      progressCalls.push(percent)
    })

    expect(hash).toBe(
      '64ec88ca00b268e5ba1a35678a1b5316d212f4f366b2477232534a8aeca37f3c',
    )
    expect(mockHasher.init).toHaveBeenCalledOnce()
    expect(mockHasher.update).toHaveBeenCalled()
    expect(progressCalls.length).toBeGreaterThan(0)
    expect(progressCalls[progressCalls.length - 1]).toBe(100)
  })

  it('falls back to Web Crypto API when WASM hasher fails', async () => {
    mockCreateSHA256.mockRejectedValueOnce(new Error('WASM blocked by CSP'))

    const content = 'Hello fallback'
    const file = new File([content], 'fallback.txt', { type: 'text/plain' })

    const hash = await calculateFileHash(file)
    // Verify hash matches Web Crypto SHA-256
    const buf = new TextEncoder().encode(content)
    const digest = await crypto.subtle.digest('SHA-256', buf)
    const expectedHash = Array.from(new Uint8Array(digest))
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('')

    expect(hash).toBe(expectedHash)
  })
})
