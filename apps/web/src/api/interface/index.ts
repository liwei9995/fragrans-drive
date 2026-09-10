// 请求响应参数(不包含data)
export interface Result {
  code: string
  msg: string
}

export interface StorageNode {
  id: string
  name: string
  baseName?: string
  extName?: string
  mimeType?: string
  encoding?: string
  size?: number
  parentId: string
  type: 'file' | 'folder'
  thumbnail?: string
  url?: string
  isPublic?: boolean
  publicSlug?: string
  publicUrl?: string
  publicExpiresAt?: string
  publicAccessCount?: number
  lastPublicAccessedAt?: string
  createdAt: string
  updatedAt: string
}

export interface StorageListResponse {
  docs: StorageNode[]
  limit: number
  page: number
  pages: number
  total?: number
}

export interface StoragePathItem {
  id: string
  name: string
}

export interface StorageFolderResult {
  exist: boolean
  id: string
  name: string
  parentId: string
}

export interface StorageUpdateResult extends StorageNode {
  exist: boolean
}

// 登录
export namespace Login {
  export interface ReqLoginForm {
    email: string
    password: string
  }
  export interface ResLogin {
    access_token: string
    refresh_token: string
  }
  export interface ResRefresh {
    access_token: string
    refresh_token: string
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
    isPublic?: boolean
    keyword?: string
    types?: string[]
    sortBy?: string
    sortOrder?: number
    viewMode?: string
    page?: number
    limit?: number
  }

  export interface ReqStorageUpdateFileName {
    name: string
    parentId: string
    type: string
  }

  export interface ReqSetPublicStatus {
    isPublic: boolean
    refresh?: boolean
    expiresIn?: number | null
  }

  export interface ResPublicStatus {
    id: string
    isPublic: boolean
    publicSlug?: string
    publicUrl?: string
    publicExpiresAt?: string
    publicAccessCount?: number
    lastPublicAccessedAt?: string
  }

  export interface TrashRestoreParams {
    fileIds?: string[]
    restoreAll?: boolean
  }

  export interface TrashRestoreResponse {
    requestedItems: number
    restoredDocs: number
  }

  export interface TrashCleanupResponse {
    deletedDocs: number
    deletedFiles: number
  }

  export interface StorageUsage {
    usedBytes: number
    fileCount: number
    quotaBytes: number
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

  export interface UserProfile {
    id: string
    email: string
    firstName: string
    lastName: string
    gender?: number
    age?: number
    avatar?: string
    roles?: string[]
    createdAt?: string
    updatedAt?: string
  }

  export interface UpdateProfileParams {
    firstName?: string
    lastName?: string
    gender?: number
    age?: number
    avatar?: string
  }

  export interface UpdatePasswordParams {
    oldPassword: string
    password: string
    changePassword: string
  }
}
