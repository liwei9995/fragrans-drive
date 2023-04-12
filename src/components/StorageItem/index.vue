<template>
	<div class="storage-item-wrapper">
		<el-image class="icon" :src="thumbUrl" fit="contain">
			<template #error>
				<div class="icon-placeholder" :style="{ backgroundImage: 'url(' + thumbPlaceholder + ')' }" />
			</template>
		</el-image>
		<span>{{ name }}</span>
	</div>
</template>

<script setup lang="ts" name="storage-item">
interface StorageItemProps {
	thumbUrl?: string
	thumbPlaceholder?: string
	name: string
}

withDefaults(defineProps<StorageItemProps>(), {
	thumbUrl: ''
})
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
