<template>
	<template v-if="isUpload">
		<Upload
			multiple
			:show-file-list="false"
			:limit="limit"
			:on-change="onUploadChange"
			:on-exceed="onUploadExceed"
			:on-progress="onUploadProgress"
			:before-upload="beforeUpload"
		>
			<template #trigger>
				<div class="empty-item-wrapper" @click="handleTap">
					<span>{{ description }}</span>
					<el-image class="icon" :src="icon" />
				</div>
			</template>
		</Upload>
	</template>
	<div v-else class="empty-item-wrapper" @click.stop="handleTap">
		<span>{{ description }}</span>
		<el-image class="icon" :src="icon" />
	</div>
</template>

<script setup lang="ts" name="empty-item">
import { UploadProps } from 'element-plus'
import Upload from '../../Upload/index.vue'

interface ItemProps {
	id: string
	description: string
	icon?: string
	isUpload?: boolean
	limit?: number
	tapItem?: (id: string) => void
	onUploadChange?: UploadProps['onChange']
	onUploadExceed?: UploadProps['onExceed']
	onUploadProgress?: UploadProps['onProgress']
	beforeUpload?: UploadProps['beforeUpload']
}

const props = withDefaults(defineProps<ItemProps>(), {
	description: () => '',
	isUpload: () => false,
	limit: () => 10,
	icon: () => 'https://img.alicdn.com/imgextra/i4/O1CN01Ojh9qS1rrJtSy0dN4_!!6000000005684-2-tps-224-224.png'
})

const handleTap = () => props.tapItem && props.tapItem(props.id)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
