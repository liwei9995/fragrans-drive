import type { User } from '@/api/interface'

/* GlobalState */
export interface GlobalState {
  accessToken: string
  userInfo?: User.UserProfile | null
  language: 'zh' | 'en'
}
