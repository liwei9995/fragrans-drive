import { Storage } from '@/api/interface/index'
import { PORT } from '@/api/config/servicePort'

import http from '@/api'

/**
 * @name 文件存储模块
 */
// 创建文件夹接口
export const createFolder = (params: Storage.ReqStorageFolder) => {
	return http.post(PORT + `/storage/folder`, params) // 正常 post json 请求  ==>  application/json
}
