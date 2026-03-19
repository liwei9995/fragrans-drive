// 请求响应参数(不包含data)
export interface Result {
  code: string
  msg: string
}

// 登录
export namespace Login {
  export interface ReqLoginForm {
    email: string
    password: string
  }
  export interface ResLogin {
    access_token: string
  }
  export interface ResAuthButtons {
    [propName: string]: unknown
  }
}

// 文件存储
export namespace Storage {
  export interface ReqStorageFolder {
    name: string
    type: string
    parentId: string
  }

  export interface ReqStorageMove {
    fileId: string
    parentId: string
  }

  export interface ReqStorageList {
    query?: Record<string, string>
    pagination?: Record<string, unknown>
  }

  export interface ReqStorageUpdateFileName {
    name: string
    parentId: string
    type: string
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
