<template>
	<div class="empty-item-wrapper" @click.stop="handleTap">
		<span>{{ description }}</span>
		<el-image class="icon" :src="icon" />
	</div>
</template>

<script setup lang="ts" name="empty-item">
interface ItemProps {
	id?: string
	description: string
	icon?: string
	tapItem?: (id?: string) => void
}

const props = withDefaults(defineProps<ItemProps>(), {
	description: () => '',
	icon: () => 'https://img.alicdn.com/imgextra/i4/O1CN01Ojh9qS1rrJtSy0dN4_!!6000000005684-2-tps-224-224.png'
})

const handleTap = () => props.tapItem && props.tapItem(props.id)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
