import { describe, expect, it, vi } from 'vitest'
import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import { ResultEnum } from '@/enums/httpEnum'
import {
  createFolder,
  deleteFile,
  emptyTrash,
  getDownloadUrl,
  getFile,
  getFiles,
  getPath,
  getStorageUsage,
  getTrashList,
  moveFile,
  restoreFile,
  restoreTrash,
  setPublicStatus,
  updateFile,
} from './storage'

vi.mock('@/api', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
    download: vi.fn(),
    put: vi.fn(),
  },
}))

describe('storage module api', () => {
  it('createFolder', () => {
    const params = { name: 'test', type: 'folder', parentId: 'root' }
    createFolder(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/folder`, params)
  })

  it('getFiles', () => {
    const params = { query: { parentId: '0' } }
    getFiles(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/list`, params)
  })

  it('deleteFile', () => {
    deleteFile('1')
    expect(http.delete).toHaveBeenCalledWith(`${PORT}/storage/1`)
  })

  it('moveFile', () => {
    const params = { fileId: '1', parentId: '2' }
    moveFile(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/move`, params)
  })

  it('getFile', () => {
    getFile('1')
    expect(http.download).toHaveBeenCalledWith(`${PORT}/storage/1`, {
      timeout: ResultEnum.TIMEOUT_DOWNLOAD,
    })
  })

  it('updateFile', () => {
    const params = { name: 'newName', parentId: 'root', type: 'file' }
    updateFile('1', params)
    expect(http.put).toHaveBeenCalledWith(`${PORT}/storage/1`, params)
  })

  it('getPath', () => {
    getPath('1')
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/path`, {
      fileId: '1',
    })
  })

  it('getDownloadUrl', () => {
    getDownloadUrl('1')
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/download/url`, {
      fileId: '1',
    })
  })

  it('setPublicStatus', () => {
    setPublicStatus('1', { isPublic: true })
    expect(http.put).toHaveBeenCalledWith(`${PORT}/storage/1/public`, {
      isPublic: true,
    })
  })

  it('getTrashList', () => {
    const params = { page: 1, limit: 20 }
    getTrashList(params)
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/trash/list`, params)
  })

  it('restoreTrash', () => {
    const params = { fileIds: ['1', '2'] }
    restoreTrash(params)
    expect(http.post).toHaveBeenCalledWith(
      `${PORT}/storage/trash/restore`,
      params,
    )
  })

  it('restoreFile', () => {
    restoreFile('1')
    expect(http.post).toHaveBeenCalledWith(`${PORT}/storage/1/restore`)
  })

  it('emptyTrash', () => {
    emptyTrash()
    expect(http.delete).toHaveBeenCalledWith(`${PORT}/storage/trash`)
  })

  it('getStorageUsage', () => {
    getStorageUsage()
    expect(http.get).toHaveBeenCalledWith(`${PORT}/storage/usage`)
  })
})
