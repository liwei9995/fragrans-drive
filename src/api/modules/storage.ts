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
