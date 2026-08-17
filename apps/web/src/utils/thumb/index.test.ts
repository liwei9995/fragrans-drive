import { describe, expect, it } from 'vitest'
import { FILE_TYPE_THUMBS, FOLDER_THUMB, getThumb } from './index'

describe('getThumb', () => {
  it('returns FOLDER_THUMB when type is folder', () => {
    expect(getThumb('.txt', 'folder')).toBe(FOLDER_THUMB)
    expect(getThumb(undefined, 'folder')).toBe(FOLDER_THUMB)
  })

  it('returns correct thumb for known file extension', () => {
    expect(getThumb('.txt', 'file')).toBe(FILE_TYPE_THUMBS['.txt'])
    expect(getThumb('.PDF', 'file')).toBe(FILE_TYPE_THUMBS['.pdf']) // test case insensitivity
    expect(getThumb('.pdf', 'file')).toBe(FILE_TYPE_THUMBS['.pdf'])
  })

  it('returns UNKNOWN thumb for unknown file extension', () => {
    expect(getThumb('.xyz', 'file')).toBe(FILE_TYPE_THUMBS.unknown)
    expect(getThumb('', 'file')).toBe(FILE_TYPE_THUMBS.unknown)
    expect(getThumb(undefined, 'file')).toBe(FILE_TYPE_THUMBS.unknown)
  })
})
