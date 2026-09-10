import http from '@/api'
import { PORT } from '@/api/config/servicePort'
import type {
  Storage,
  StorageFolderResult,
  StorageListResponse,
  StoragePathItem,
  StorageUpdateResult,
} from '@/api/interface/index'
import { ResultEnum } from '@/enums/httpEnum'

/**
 * @name 文件存储模块
 */
// 创建文件夹接口
export const createFolder = (params: Storage.ReqStorageFolder) => {
  return http.post<StorageFolderResult>(`${PORT}/storage/folder`, params)
}

// 获取文件/文件夹列表接口
export const getFiles = (params?: Storage.ReqStorageList) => {
  return http.post<StorageListResponse>(`${PORT}/storage/list`, params)
}

// 删除文件/文件夹接口
export const deleteFile = (id: string) => {
  return http.delete<void>(`${PORT}/storage/${id}`)
}

// 移动文件/文件夹接口
export const moveFile = (params: Storage.ReqStorageMove) => {
  return http.post<void>(`${PORT}/storage/move`, params)
}

// 获取文件接口
export const getFile = (id: string) => {
  return http.download(`${PORT}/storage/${id}`, {
    timeout: ResultEnum.TIMEOUT_DOWNLOAD as number,
  })
}

// 修改文件/文件夹信息接口
export const updateFile = (
  id: string,
  params: Storage.ReqStorageUpdateFileName,
) => {
  return http.put<StorageUpdateResult>(`${PORT}/storage/${id}`, params)
}

// 获取文件/文件夹所在路径
export const getPath = (fileId: string) => {
  return http.post<StoragePathItem[]>(`${PORT}/storage/path`, { fileId })
}

// 获取带 Download 票的文件下载地址
export const getDownloadUrl = (fileId: string) => {
  return http.post<string>(`${PORT}/storage/download/url`, { fileId })
}

// 设置文件公开直链状态
export const setPublicStatus = (
  id: string,
  params: Storage.ReqSetPublicStatus,
) => {
  return http.put<Storage.ResPublicStatus>(
    `${PORT}/storage/${id}/public`,
    params,
  )
}

// 获取回收站列表
export const getTrashList = (params?: Storage.ReqStorageList) => {
  return http.post<StorageListResponse>(`${PORT}/storage/trash/list`, params)
}

// 恢复指定或全部回收站文件
export const restoreTrash = (params: Storage.TrashRestoreParams) => {
  return http.post<Storage.TrashRestoreResponse>(
    `${PORT}/storage/trash/restore`,
    params,
  )
}

// 恢复单个文件
export const restoreFile = (id: string) => {
  return http.post<StorageUpdateResult>(`${PORT}/storage/${id}/restore`)
}

// 清空回收站
export const emptyTrash = () => {
  return http.delete<Storage.TrashCleanupResponse>(`${PORT}/storage/trash`)
}

// 获取存储容量与文件统计
export const getStorageUsage = () => {
  return http.get<Storage.StorageUsage>(`${PORT}/storage/usage`)
}
