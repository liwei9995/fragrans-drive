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
 * @name File storage module
 */
// Create folder endpoint
export const createFolder = (params: Storage.ReqStorageFolder) => {
  return http.post<StorageFolderResult>(`${PORT}/storage/folder`, params)
}

// Get file/folder list endpoint
export const getFiles = (params?: Storage.ReqStorageList) => {
  return http.post<StorageListResponse>(`${PORT}/storage/list`, params)
}

// Delete file/folder endpoint
export const deleteFile = (id: string) => {
  return http.delete<void>(`${PORT}/storage/${id}`)
}

// Move file/folder endpoint
export const moveFile = (params: Storage.ReqStorageMove) => {
  return http.post<void>(`${PORT}/storage/move`, params)
}

// Get file download stream endpoint
export const getFile = (id: string) => {
  return http.download(`${PORT}/storage/${id}`, {
    timeout: ResultEnum.TIMEOUT_DOWNLOAD as number,
  })
}

// Update file/folder information endpoint
export const updateFile = (
  id: string,
  params: Storage.ReqStorageUpdateFileName,
) => {
  return http.put<StorageUpdateResult>(`${PORT}/storage/${id}`, params)
}

// Get file/folder path endpoint
export const getPath = (fileId: string) => {
  return http.post<StoragePathItem[]>(`${PORT}/storage/path`, { fileId })
}

// Get file download URL with ticket
export const getDownloadUrl = (fileId: string) => {
  return http.post<string>(`${PORT}/storage/download/url`, { fileId })
}

// Set file public direct link status
export const setPublicStatus = (
  id: string,
  params: Storage.ReqSetPublicStatus,
) => {
  return http.put<Storage.ResPublicStatus>(
    `${PORT}/storage/${id}/public`,
    params,
  )
}

// Get trash list endpoint
export const getTrashList = (params?: Storage.ReqStorageList) => {
  return http.post<StorageListResponse>(`${PORT}/storage/trash/list`, params)
}

// Restore specified or all files from trash
export const restoreTrash = (params: Storage.TrashRestoreParams) => {
  return http.post<Storage.TrashRestoreResponse>(
    `${PORT}/storage/trash/restore`,
    params,
  )
}

// Restore single file from trash
export const restoreFile = (id: string) => {
  return http.post<StorageUpdateResult>(`${PORT}/storage/${id}/restore`)
}

// Empty trash endpoint
export const emptyTrash = () => {
  return http.delete<Storage.TrashCleanupResponse>(`${PORT}/storage/trash`)
}

// Permanently delete specified files from trash
export const deleteTrash = (params: Storage.TrashDeleteParams) => {
  return http.post<Storage.TrashCleanupResponse>(
    `${PORT}/storage/trash/delete`,
    params,
  )
}

// Get storage usage and file statistics
export const getStorageUsage = () => {
  return http.get<Storage.StorageUsage>(`${PORT}/storage/usage`)
}
