import type { UploadFile, UploadProgressEvent } from 'element-plus'
import { describe, expect, it, vi } from 'vitest'
import { useUploadQueue } from './useUploadQueue'

const file = (status: UploadFile['status'], percentage = 100): UploadFile => ({
  name: 'file.txt',
  percentage,
  size: 100,
  status,
  uid: 1,
})

describe('useUploadQueue', () => {
  it('tracks progress and completes a successful batch once', () => {
    const onChange = vi.fn()
    const onComplete = vi.fn()
    const showStatus = vi.fn()
    const queue = useUploadQueue({ onChange, onComplete, showStatus })
    const uploading = file('uploading', 50)

    queue.handleUploadProgress({} as UploadProgressEvent, uploading, [
      uploading,
    ])
    expect(queue.uploadPercentage.value).toBe(50)

    const success = file('success')
    queue.handleUploadChange(success, [success])

    expect(queue.notificationTitle.value).toBe('上传完成 ∙ 共1项')
    expect(queue.notificationType.value).toBe('success')
    expect(showStatus).toHaveBeenCalledOnce()
    expect(onChange).toHaveBeenCalledOnce()
    expect(onComplete).toHaveBeenCalledOnce()
    expect(queue.uploadedFiles.value).toEqual([])

    queue.handleUploadChange(success, [success])
    expect(onComplete).toHaveBeenCalledOnce()
  })

  it('reports pending and failed files in the same batch', () => {
    const onComplete = vi.fn()
    const queue = useUploadQueue({
      onChange: vi.fn(),
      onComplete,
      showStatus: vi.fn(),
    })
    const success = file('success')
    const uploading = { ...file('uploading', 25), uid: 2 }

    queue.handleUploadChange(success, [success, uploading])
    expect(queue.notificationTitle.value).toBe('正在上传 ∙ 剩余1项')
    expect(queue.notificationType.value).toBe('uploading')

    const failed = file('fail')
    queue.handleUploadChange(failed, [failed])
    expect(queue.notificationTitle.value).toBe('上传完成 ∙ 成功0项 失败1项')
    expect(queue.notificationType.value).toBe('error')
    expect(onComplete).toHaveBeenCalledOnce()
  })

  it('ignores an empty change and reports zero progress for empty files', () => {
    const showStatus = vi.fn()
    const queue = useUploadQueue({
      onChange: vi.fn(),
      onComplete: vi.fn(),
      showStatus,
    })
    const ready = file('ready')

    queue.handleUploadChange(ready, [])
    queue.handleUploadProgress({} as UploadProgressEvent, ready, [
      { ...ready, size: 0 },
    ])

    expect(showStatus).not.toHaveBeenCalled()
    expect(queue.uploadPercentage.value).toBe(0)
  })
})
