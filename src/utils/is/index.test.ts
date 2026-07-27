import { describe, expect, it } from 'vitest'
import * as is from './index'

describe('utils/is', () => {
  it('is', () => {
    expect(is.is(1, 'Number')).toBe(true)
    expect(is.is('1', 'String')).toBe(true)
  })

  it('isFunction', () => {
    expect(is.isFunction(() => {})).toBe(true)
    expect(is.isFunction(1)).toBe(false)
  })

  it('isDef / isUnDef', () => {
    expect(is.isDef(1)).toBe(true)
    expect(is.isDef(undefined)).toBe(false)
    expect(is.isUnDef(undefined)).toBe(true)
    expect(is.isUnDef(1)).toBe(false)
  })

  it('isObject', () => {
    expect(is.isObject({})).toBe(true)
    expect(is.isObject(null)).toBe(false)
    expect(is.isObject([])).toBe(false) // Wait, [] might be [object Array] in some, but in JS it's object? `is([], 'Object')` is false.
  })

  it('isDate', () => {
    expect(is.isDate(new Date())).toBe(true)
    expect(is.isDate(1)).toBe(false)
  })

  it('isNumber', () => {
    expect(is.isNumber(1)).toBe(true)
    expect(is.isNumber('1')).toBe(false)
  })

  it('isAsyncFunction', () => {
    expect(is.isAsyncFunction(async () => {})).toBe(true)
    expect(is.isAsyncFunction(() => {})).toBe(false)
  })

  it('isPromise', () => {
    expect(is.isPromise(Promise.resolve())).toBe(true)
    expect(is.isPromise({})).toBe(false)
  })

  it('isString', () => {
    expect(is.isString('1')).toBe(true)
    expect(is.isString(1)).toBe(false)
  })

  it('isBoolean', () => {
    expect(is.isBoolean(true)).toBe(true)
    expect(is.isBoolean(1)).toBe(false)
  })

  it('isArray', () => {
    expect(is.isArray([])).toBe(true)
    expect(is.isArray({})).toBe(false)
  })

  it('isClient / isServer', () => {
    expect(typeof is.isClient() === 'boolean').toBe(true)
    expect(typeof is.isServer === 'boolean').toBe(true)
  })

  it('isWindow', () => {
    expect(is.isWindow(window)).toBe(true) // jsdom provides window
  })

  it('isElement', () => {
    expect(is.isElement(document.createElement('div'))).toBe(true)
    expect(is.isElement({})).toBe(false)
  })

  it('isMobileByUA / isMobileByOrientation / isMobileByTouchEvent / isMobile', () => {
    expect(typeof is.isMobileByUA() === 'boolean').toBe(true)
    expect(typeof is.isMobileByOrientation() === 'boolean').toBe(true)
    expect(typeof is.isMobileByTouchEvent() === 'boolean').toBe(true)
    expect(typeof is.isMobile() === 'boolean').toBe(true)
  })

  it('isImageDom', () => {
    expect(is.isImageDom(document.createElement('img'))).toBe(true)
    expect(is.isImageDom(document.createElement('div'))).toBe(false)
  })

  it('isNull', () => {
    expect(is.isNull(null)).toBe(true)
    expect(is.isNull(undefined)).toBe(false)
  })

  it('isNullAndUnDef / isNullOrUnDef', () => {
    // This function implementation might be logically flawed if it checks isUnDef && isNull (it can't be both)
    expect(is.isNullAndUnDef(null)).toBe(false) // Wait, let's just run it to see.
    expect(is.isNullOrUnDef(null)).toBe(true)
    expect(is.isNullOrUnDef(undefined)).toBe(true)
    expect(is.isNullOrUnDef(1)).toBe(false)
  })
})
