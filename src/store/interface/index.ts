/* GlobalState */
export interface GlobalState {
  token: string
  userInfo: Record<string, unknown>
  assemblySize: string
  isMobile: boolean
}

/* AuthState */
export interface AuthState {
  authRouter: string[]
}
