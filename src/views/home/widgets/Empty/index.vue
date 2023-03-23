<template>
	<div class="empty-placeholder">
		<div class="items">
			<EmptyItem
				v-for="item in items"
				:key="item.id"
				:id="item.id"
				:description="item.description"
				:icon="item.icon"
				:tap-item="handleTapItem"
			/>
		</div>
	</div>
</template>

<script setup lang="ts" name="empty">
import EmptyItem from './Item/index.vue'

const items = [
	{
		id: 'createFolder',
		description: '新建文件夹',
		icon: 'https://img.alicdn.com/imgextra/i2/O1CN018yXBXY1caApf7qUew_!!6000000003616-2-tps-224-224.png'
	},
	{
		id: 'uploadFile',
		description: '上传文件',
		icon: 'https://img.alicdn.com/imgextra/i4/O1CN01Ojh9qS1rrJtSy0dN4_!!6000000005684-2-tps-224-224.png'
	}
]

const handleTapItem = (id?: string) => console.log(id)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
