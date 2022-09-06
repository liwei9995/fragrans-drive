// 请求响应参数(不包含data)
export interface Result {
	code: string
	msg: string
}

// 登录
export namespace Login {
	export interface ReqLoginForm {
		username: string
		password: string
	}
	export interface ResLogin {
		access_token: string
	}
	export interface ResAuthButtons {
		[propName: string]: any
	}
}

// 文件存储
export namespace Storage {
	export interface ReqStorageFolder {
		name: string
		type: string
		parentId: string
	}
}

// * 用户管理模块
export namespace User {
	export interface ResUserList {
		userId: string
		email: string
		gender: string
		age: number
		address: string
		createdAt: string
		updatedAt: string
		avatar: string
		roles: string[]
	}
}
