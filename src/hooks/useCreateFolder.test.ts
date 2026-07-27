import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useCreateFolder } from './useCreateFolder'
import { ElMessage } from 'element-plus'
import * as storageApi from '@/api/modules/storage'

vi.mock('element-plus', () => ({
  ElMessage: {
    info: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    closeAll: vi.fn()
  }
}))

vi.mock('@/api/modules/storage', () => ({
  createFolder: vi.fn()
}))

describe('useCreateFolder', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should create folder successfully', async () => {
    const onSuccess = vi.fn()
    const mockRes = { exist: false, name: 'test', id: '1', parentId: '0' }
    vi.spyOn(storageApi, 'createFolder').mockResolvedValue(mockRes)

    useCreateFolder('test', '0', onSuccess)
    
    expect(ElMessage.info).toHaveBeenCalledWith({ message: '正在创建文件夹...', duration: 0 })
    expect(storageApi.createFolder).toHaveBeenCalledWith({ name: 'test', type: 'folder', parentId: '0' })

    // Wait for promise resolution
    await vi.waitFor(() => {
      expect(ElMessage.closeAll).toHaveBeenCalled()
    })
    
    expect(ElMessage.success).toHaveBeenCalledWith('创建成功')
    expect(onSuccess).toHaveBeenCalledWith({ name: 'test', id: '1', parentId: '0' })
  })

  it('should show error if folder already exists', async () => {
    const onSuccess = vi.fn()
    const mockRes = { exist: true, name: 'test', id: '1', parentId: '0' }
    vi.spyOn(storageApi, 'createFolder').mockResolvedValue(mockRes)

    useCreateFolder('test', '0', onSuccess)

    await vi.waitFor(() => {
      expect(ElMessage.closeAll).toHaveBeenCalled()
    })
    
    expect(ElMessage.error).toHaveBeenCalledWith('此目录下已存在同名文件，请修改名称')
    expect(onSuccess).not.toHaveBeenCalled()
  })

  it('should show error if createFolder fails', async () => {
    vi.spyOn(storageApi, 'createFolder').mockRejectedValue(new Error('fail'))

    useCreateFolder('test', '0')

    await vi.waitFor(() => {
      expect(ElMessage.error).toHaveBeenCalledWith('创建失败，请重试')
    })
  })
})
