const FILE_TYPE_TXT = '.txt'
const FILE_TYPE_PDF = '.pdf'
const FILE_TYPE_JS = '.js'
const FILE_TYPE_JSON = '.json'
const FILE_TYPE_MARKDOWN = '.md'
const FILE_TYPE_UNKNOWN = 'unknown'

const FOLDER_THUMB = 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png'

export const FILE_TYPE_THUMBS = {
	[FILE_TYPE_TXT]: 'https://img.alicdn.com/imgextra/i2/O1CN01WfrXxI1PzHYGt1CQo_!!6000000001911-2-tps-140-140.png',
	[FILE_TYPE_PDF]: 'https://img.alicdn.com/imgextra/i4/O1CN016J66R728rBF4EJ8Ml_!!6000000007985-2-tps-140-140.png',
	[FILE_TYPE_JS]: 'https://img.alicdn.com/imgextra/i1/O1CN013zMVdx25JnWnWO9cN_!!6000000007506-2-tps-140-140.png',
	[FILE_TYPE_JSON]: 'https://img.alicdn.com/imgextra/i4/O1CN01E88OQe1JrbCSYAAZt_!!6000000001082-2-tps-140-140.png',
	[FILE_TYPE_MARKDOWN]: 'https://img.alicdn.com/imgextra/i1/O1CN013Vi8M91rnej9zTLrG_!!6000000005676-2-tps-140-140.png',
	[FILE_TYPE_UNKNOWN]: 'https://img.alicdn.com/imgextra/i1/O1CN01NVSzRz25VFRGlsewQ_!!6000000007531-2-tps-140-140.png'
}

export const getThumb = (extName = '', type = 'file') => {
	if (type === 'folder') {
		return FOLDER_THUMB
	} else {
		return FILE_TYPE_THUMBS[extName as keyof typeof FILE_TYPE_THUMBS] || FILE_TYPE_THUMBS[FILE_TYPE_UNKNOWN]
	}
}
