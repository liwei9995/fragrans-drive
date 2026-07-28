import { ElMessage } from 'element-plus'
import { describe, expect, it, vi } from 'vitest'
import { checkStatus } from './checkStatus'

vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
  },
}))

describe('checkStatus', () => {
  it('handles 400', () => {
    checkStatus(400)
    expect(ElMessage.error).toHaveBeenCalledWith('请求失败！请您稍后重试')
  })
  it('handles 401', () => {
    checkStatus(401)
    expect(ElMessage.error).toHaveBeenCalledWith('登录失效！请您重新登录')
  })
  it('handles 403', () => {
    checkStatus(403)
    expect(ElMessage.error).toHaveBeenCalledWith('当前账号无权限访问！')
  })
  it('handles 404', () => {
    checkStatus(404)
    expect(ElMessage.error).toHaveBeenCalledWith('你所访问的资源不存在！')
  })
  it('handles 405', () => {
    checkStatus(405)
    expect(ElMessage.error).toHaveBeenCalledWith('请求方式错误！请您稍后重试')
  })
  it('handles 408', () => {
    checkStatus(408)
    expect(ElMessage.error).toHaveBeenCalledWith('请求超时！请您稍后重试')
  })
  it('handles 500', () => {
    checkStatus(500)
    expect(ElMessage.error).toHaveBeenCalledWith('服务异常！')
  })
  it('handles 502', () => {
    checkStatus(502)
    expect(ElMessage.error).toHaveBeenCalledWith('网关错误！')
  })
  it('handles 503', () => {
    checkStatus(503)
    expect(ElMessage.error).toHaveBeenCalledWith('服务不可用！')
  })
  it('handles 504', () => {
    checkStatus(504)
    expect(ElMessage.error).toHaveBeenCalledWith('网关超时！')
  })
  it('handles other', () => {
    checkStatus(999)
    expect(ElMessage.error).toHaveBeenCalledWith('请求失败！')
  })
})
