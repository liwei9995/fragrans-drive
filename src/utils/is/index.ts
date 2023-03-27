const toString = Object.prototype.toString

/**
 * @description: 判断值是否未某个类型
 */
export const is = (val: unknown, type: string) => toString.call(val) === `[object ${type}]`

/**
 * @description:  是否为函数
 */
export const isFunction = <T = Function>(val: unknown): val is T => is(val, 'Function')

/**
 * @description: 是否已定义
 */
export const isDef = <T = unknown>(val?: T): val is T => typeof val !== 'undefined'

export const isUnDef = <T = unknown>(val?: T): val is T => !isDef(val)

/**
 * @description: 是否为对象
 */
export const isObject = (val: any): val is Record<any, any> => val !== null && is(val, 'Object')

/**
 * @description:  是否为时间
 */
export const isDate = (val: unknown): val is Date => is(val, 'Date')

/**
 * @description:  是否为数值
 */
export const isNumber = (val: unknown): val is number => is(val, 'Number')

/**
 * @description:  是否为AsyncFunction
 */
export const isAsyncFunction = <T = any>(val: unknown): val is Promise<T> => is(val, 'AsyncFunction')

/**
 * @description:  是否为promise
 */
export const isPromise = <T = any>(val: unknown): val is Promise<T> =>
	is(val, 'Promise') && isObject(val) && isFunction(val.then) && isFunction(val.catch)

/**
 * @description:  是否为字符串
 */
export const isString = (val: unknown): val is string => is(val, 'String')

/**
 * @description:  是否为boolean类型
 */
export const isBoolean = (val: unknown): val is boolean => is(val, 'Boolean')

/**
 * @description:  是否为数组
 */
export const isArray = (val: any): val is Array<any> => val && Array.isArray(val)

/**
 * @description: 是否客户端
 */
export const isClient = () => typeof window !== 'undefined'

/**
 * @description: 是否为浏览器
 */
export const isWindow = (val: any): val is Window => typeof window !== 'undefined' && is(val, 'Window')

export const isElement = (val: unknown): val is Element => isObject(val) && !!val.tagName

export const isServer = typeof window === 'undefined'

export const isMobileByUA = () => isWindow(window) && /Mobi|Android|iPhone/i.test(navigator?.userAgent)

export const isMobileByOrientation = () => isWindow(window) && typeof window.orientation !== 'undefined'

export const isMobileByTouchEvent = () => isWindow(window) && 'ontouchstart' in document.documentElement

export const isMobile = () => isMobileByUA() || isMobileByOrientation() || isMobileByTouchEvent()

// 是否为图片节点
export const isImageDom = (o: Element) => o && ['IMAGE', 'IMG'].includes(o.tagName)

export const isNull = (val: unknown): val is null => val === null

export const isNullAndUnDef = (val: unknown): val is null | undefined => isUnDef(val) && isNull(val)

export const isNullOrUnDef = (val: unknown): val is null | undefined => isUnDef(val) || isNull(val)
