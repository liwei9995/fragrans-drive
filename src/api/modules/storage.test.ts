import { describe, it, expect, vi } from 'vitest'
import { createFolder, getFiles, deleteFile, moveFile, getFile, updateFile, getPath, getDownloadUrl } from './storage'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import { ResultEnum } from '@/enums/httpEnum'

vi.mock('@/api', () => ({
  default: {
    post: vi.fn(),
    delete: vi.fn(),
    download: vi.fn(),
    put: vi.fn(),
  }
}))

describe('storage module api', () => {
  it('createFolder', () => {
    const params = { name: 'test', type: 'folder' } as any
    createFolder(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/folder`, params)
  })

  it('getFiles', () => {
    const params = { query: { parentId: '0' } } as any
    getFiles(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/list`, params)
  })

  it('deleteFile', () => {
    deleteFile('1')
    expect(http.delete).toHaveBeenCalledWith(`${PORT}/storage/1`)
  })

  it('moveFile', () => {
    const params = { targetId: '2', fileIds: ['1'] } as any
    moveFile(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/move`, params)
  })

  it('getFile', () => {
    getFile('1')
    expect(http.download).toHaveBeenCalledWith(`${PORT}/storage/1`, { timeout: ResultEnum.TIMEOUT_DOWNLOAD })
  })

  it('updateFile', () => {
    const params = { name: 'newName' } as any
    updateFile('1', params)
    expect(http.put).toHaveBeenCalledWith(`${PORT}/storage/1`, params)
  })

  it('getPath', () => {
    getPath('1')
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/path`, { fileId: '1' })
  })

  it('getDownloadUrl', () => {
    getDownloadUrl('1')
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/download/url`, { fileId: '1' })
  })
})
