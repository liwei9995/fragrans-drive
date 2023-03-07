const FILE_TYPE_TXT = '.txt'
const FILE_TYPE_PDF = '.pdf'
const FILE_TYPE_JS = '.js'
const FILE_TYPE_JSON = '.json'
const FILE_TYPE_MARKDOWN = '.md'
const FILE_TYPE_HTML = '.html'
const FILE_TYPE_HTM = '.htm'
const FILE_TYPE_NUMBERS = '.numbers'
const FILE_TYPE_CSV = '.csv'
const FILE_TYPE_DOC = '.doc'
const FILE_TYPE_DOCX = '.docx'
const FILE_TYPE_XLS = '.xls'
const FILE_TYPE_XLSX = '.xlsx'
const FILE_TYPE_PPT = '.ppt'
const FILE_TYPE_PPTX = '.pptx'
const FILE_TYPE_PAGES = '.pages'
const FILE_TYPE_ZIP = '.zip'
const FILE_TYPE_7Z = '.7z'
const FILE_TYPE_TAR = '.tar'
const FILE_TYPE_GZ = '.gz'
const FILE_TYPE_RAR = '.rar'
const FILE_TYPE_KEY = '.key'
const FILE_TYPE_SVG = '.svg'
const FILE_TYPE_MP4 = '.mp4'
const FILE_TYPE_AVI = '.avi'
const FILE_TYPE_MOV = '.mov'
const FILE_TYPE_WMV = '.wmv'
const FILE_TYPE_FLV = '.flv'
const FILE_TYPE_MKV = '.mkv'
const FILE_TYPE_UNKNOWN = 'unknown'

export const FOLDER_THUMB = 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png'

export const FILE_TYPE_THUMBS = {
	[FILE_TYPE_TXT]: 'https://img.alicdn.com/imgextra/i2/O1CN01WfrXxI1PzHYGt1CQo_!!6000000001911-2-tps-140-140.png',
	[FILE_TYPE_PDF]: 'https://img.alicdn.com/imgextra/i4/O1CN016J66R728rBF4EJ8Ml_!!6000000007985-2-tps-140-140.png',
	[FILE_TYPE_JS]: 'https://img.alicdn.com/imgextra/i1/O1CN013zMVdx25JnWnWO9cN_!!6000000007506-2-tps-140-140.png',
	[FILE_TYPE_JSON]: 'https://img.alicdn.com/imgextra/i4/O1CN01E88OQe1JrbCSYAAZt_!!6000000001082-2-tps-140-140.png',
	[FILE_TYPE_HTM]: 'https://img.alicdn.com/imgextra/i4/O1CN01SWxi4l1rNY9SoHzkz_!!6000000005619-2-tps-140-140.png',
	[FILE_TYPE_HTML]: 'https://img.alicdn.com/imgextra/i4/O1CN01SWxi4l1rNY9SoHzkz_!!6000000005619-2-tps-140-140.png',
	[FILE_TYPE_MARKDOWN]: 'https://img.alicdn.com/imgextra/i1/O1CN013Vi8M91rnej9zTLrG_!!6000000005676-2-tps-140-140.png',
	[FILE_TYPE_NUMBERS]: 'https://img.alicdn.com/imgextra/i2/O1CN01jQg56D1zMBtsuwn6U_!!6000000006699-2-tps-140-140.png',
	[FILE_TYPE_CSV]: 'https://img.alicdn.com/imgextra/i3/O1CN01seltan1FMDbHjtByl_!!6000000000472-2-tps-140-140.png',
	[FILE_TYPE_DOC]: 'https://img.alicdn.com/imgextra/i2/O1CN017vpxdQ27S9zPCPqMD_!!6000000007795-2-tps-140-140.png',
	[FILE_TYPE_DOCX]: 'https://img.alicdn.com/imgextra/i2/O1CN017vpxdQ27S9zPCPqMD_!!6000000007795-2-tps-140-140.png',
	[FILE_TYPE_XLS]: 'https://img.alicdn.com/imgextra/i3/O1CN01seltan1FMDbHjtByl_!!6000000000472-2-tps-140-140.png',
	[FILE_TYPE_XLSX]: 'https://img.alicdn.com/imgextra/i3/O1CN01seltan1FMDbHjtByl_!!6000000000472-2-tps-140-140.png',
	[FILE_TYPE_PPT]: 'https://img.alicdn.com/imgextra/i2/O1CN01rCYy6x1tYezHtMNq2_!!6000000005914-2-tps-140-140.png',
	[FILE_TYPE_PPTX]: 'https://img.alicdn.com/imgextra/i2/O1CN01rCYy6x1tYezHtMNq2_!!6000000005914-2-tps-140-140.png',
	[FILE_TYPE_PAGES]: 'https://img.alicdn.com/imgextra/i2/O1CN019Ed3Cq1cdq00SKcPY_!!6000000003624-2-tps-140-140.png',
	[FILE_TYPE_ZIP]: 'https://img.alicdn.com/imgextra/i3/O1CN01iGK86t1XF2UJeHczd_!!6000000002893-2-tps-140-140.png',
	[FILE_TYPE_7Z]: 'https://img.alicdn.com/imgextra/i3/O1CN01iGK86t1XF2UJeHczd_!!6000000002893-2-tps-140-140.png',
	[FILE_TYPE_TAR]: 'https://img.alicdn.com/imgextra/i3/O1CN01iGK86t1XF2UJeHczd_!!6000000002893-2-tps-140-140.png',
	[FILE_TYPE_GZ]: 'https://img.alicdn.com/imgextra/i3/O1CN01iGK86t1XF2UJeHczd_!!6000000002893-2-tps-140-140.png',
	[FILE_TYPE_RAR]: 'https://img.alicdn.com/imgextra/i3/O1CN01iGK86t1XF2UJeHczd_!!6000000002893-2-tps-140-140.png',
	[FILE_TYPE_KEY]: 'https://img.alicdn.com/imgextra/i1/O1CN01obpWj31QWG3V8dpU8_!!6000000001983-2-tps-140-140.png',
	[FILE_TYPE_SVG]: 'https://img.alicdn.com/imgextra/i4/O1CN01LJkSQb1rZurCPWBE2_!!6000000005646-2-tps-140-140.png',
	[FILE_TYPE_MP4]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_AVI]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_MOV]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_WMV]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_FLV]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_MKV]: 'https://img.alicdn.com/imgextra/i4/O1CN01FkWoEz1Q5EhTaCJfg_!!6000000001924-2-tps-140-140.png',
	[FILE_TYPE_UNKNOWN]: 'https://img.alicdn.com/imgextra/i1/O1CN01NVSzRz25VFRGlsewQ_!!6000000007531-2-tps-140-140.png'
}

export const getThumb = (extName = '', type = 'file') => {
	if (type === 'folder') {
		return FOLDER_THUMB
	} else {
		const lowerCaseExtName = extName?.toLowerCase()

		return FILE_TYPE_THUMBS[lowerCaseExtName as keyof typeof FILE_TYPE_THUMBS] || FILE_TYPE_THUMBS[FILE_TYPE_UNKNOWN]
	}
}
