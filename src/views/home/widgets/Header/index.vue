<template>
	<header class="header">
		<div class="page-title">
			<el-breadcrumb separator="›">
				<template v-for="item in breadcrumbItems" :key="item.id">
					<el-breadcrumb-item v-if="!item.isOmit && !item.isHighlight">
						<div class="breadcrumb-item-content">{{ item.text }}</div>
					</el-breadcrumb-item>
					<el-breadcrumb-item v-else-if="item.isOmit">
						<el-icon :size="12">
							<MoreFilled />
						</el-icon>
					</el-breadcrumb-item>
					<div class="breadcrumb-item-content highlight" v-else-if="item.isHighlight">
						{{ item.text }}
					</div>
				</template>
			</el-breadcrumb>
		</div>
		<div class="actions">
			<div class="search">
				<el-icon :size="24">
					<Search />
				</el-icon>
			</div>
			<el-dropdown>
				<div class="action">
					<el-icon :size="32">
						<CirclePlusFilled />
					</el-icon>
				</div>
				<template #dropdown>
					<el-dropdown-menu>
						<el-dropdown-item v-for="item in actionItems" :key="item.id">
							{{ item.name }}
						</el-dropdown-item>
					</el-dropdown-menu>
				</template>
			</el-dropdown>
		</div>
	</header>
</template>

<script setup lang="ts" name="header">
type BreadcrumbItem = {
	id?: string
	text?: string
	isHighlight?: boolean
	isOmit?: boolean
}

type ActionItem = {
	id?: string
	name: string
	isUpload?: boolean
}

interface HeaderProps {
	breadcrumbItems?: Partial<BreadcrumbItem>[]
	actionItems?: Partial<ActionItem>[]
}

withDefaults(defineProps<HeaderProps>(), {
	breadcrumbItems: () => [
		{
			id: '1',
			text: '文件'
		},
		{
			id: '2',
			isOmit: true
		},
		{
			id: '3',
			isHighlight: true,
			text: 'NBA录像'
		}
	],
	actionItems: () => [
		{
			id: '1',
			name: '新建文件夹'
		},
		{
			id: '2',
			name: '上传文件',
			isUpload: true
		}
	]
})
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
