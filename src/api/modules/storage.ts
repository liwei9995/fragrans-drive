import { Storage } from '@/api/interface/index'
import { PORT } from '@/api/config/servicePort'

import http from '@/api'

/**
 * @name 文件存储模块
 */
// 创建文件夹接口
export const createFolder = (params: Storage.ReqStorageFolder) => {
	return http.post(`${PORT}/storage/folder`, params) // 正常 post json 请求  ==>  application/json
}

// 获取文件/文件夹列表接口
export const getFiles = (params?: Storage.ReqStorageList) => {
	return http.post(`${PORT}/storage/list`, params) // 正常 post json 请求  ==>  application/json
}

// 获取文件接口
export const getFile = (id: string) => {
	return http.get(`${PORT}/storage/${id}`)
}
