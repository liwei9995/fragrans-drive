import { ElMessage } from 'element-plus'
import { createFolder } from '@/api/modules/storage'

export const useCreateFolder = (name: string, parentId: string, onSuccess?: () => void) => {
	ElMessage.info({
		message: '正在创建文件夹...',
		duration: 0
	})

	createFolder({
		name,
		type: 'folder',
		parentId
	})
		.then(({ exist }) => {
			ElMessage.closeAll()

			if (exist) {
				ElMessage.error('此目录下已存在同名文件，请修改名称')
			} else {
				ElMessage.success('创建成功')
				typeof onSuccess === 'function' && onSuccess()
			}
		})
		.catch(() => ElMessage.error('创建失败，请重试'))
}
