// Request response parameters (excluding data)
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
  contentHash?: string
  md5Hash?: string
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

// Authentication
export namespace Login {
  export interface ReqLoginForm {
    email: string
    password: string
  }
  export interface ResLogin {
    access_token: string
  }
  export interface ResRefresh {
    access_token: string
  }
  export interface ResAuthButtons {
    [propName: string]: unknown
  }
}

// File storage
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
    parentId?: string
    type?: string
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

  export interface TrashDeleteParams {
    fileIds: string[]
  }

  export interface StorageUsage {
    usedBytes: number
    fileCount: number
    quotaBytes: number
  }
}

// * User management module
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

// * Authentication and security module
export namespace Auth {
  export interface AuthConfig {
    allowRegistration: boolean
    emailVerificationRequired: boolean
    captchaRequired: boolean
  }

  export interface CaptchaData {
    id: string
    svg: string
  }

  export interface ReqSendEmailCode {
    email: string
    purpose: 'register' | 'reset_password'
    captchaId: string
    captchaCode: string
  }

  export interface ReqRegister {
    email: string
    password: string
    firstName: string
    lastName: string
    captchaId?: string
    captchaCode?: string
    emailCode?: string
  }

  export interface ReqResetPassword {
    email: string
    code: string
    password: string
    changePassword: string
  }
}
