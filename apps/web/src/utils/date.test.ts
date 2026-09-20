import { describe, expect, it } from 'vitest'
import { formatDateTime, formatLocaleDate, parseDate } from './date'

describe('date utils', () => {
  it('parses numeric timestamp strings in ms', () => {
    const d = parseDate('1789889500000')
    expect(d).not.toBeNull()
    expect(d?.getTime()).toBe(1789889500000)
  })

  it('parses numeric timestamp strings in seconds', () => {
    const d = parseDate('1789889500')
    expect(d).not.toBeNull()
    expect(d?.getTime()).toBe(1789889500000)
  })

  it('parses numbers', () => {
    const d = parseDate(1789889500000)
    expect(d).not.toBeNull()
    expect(d?.getTime()).toBe(1789889500000)
  })

  it('parses ISO date strings', () => {
    const d = parseDate('2026-09-20T07:35:00.000Z')
    expect(d).not.toBeNull()
    expect(d?.toISOString()).toBe('2026-09-20T07:35:00.000Z')
  })

  it('returns null for invalid or empty dates', () => {
    expect(parseDate(undefined)).toBeNull()
    expect(parseDate(null)).toBeNull()
    expect(parseDate('')).toBeNull()
    expect(parseDate('not-a-date')).toBeNull()
  })

  it('formats locale date safely', () => {
    const formatted = formatLocaleDate('1789889500000')
    expect(formatted).not.toBe('Invalid Date')
    expect(formatted).not.toBe('Unknown')

    expect(formatLocaleDate(undefined)).toBe('Unknown')
    expect(formatLocaleDate('invalid')).toBe('Unknown')
  })

  it('formats date time safely', () => {
    const formatted = formatDateTime('2026-09-20T07:35:00.000Z')
    expect(formatted).not.toBe('-')
    expect(formatDateTime(undefined)).toBe('-')
  })
})
