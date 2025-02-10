<script setup lang="ts" name="breadcrumb">
import { useRouter } from 'vue-router'
import { HOME_URL } from '@/config/config'

export type BreadcrumbItem = {
	id?: string
	text?: string
	isHighlight?: boolean
	isOmit?: boolean
}

interface BreadcrumbProps {
	autoNav?: boolean
	breadcrumbItems?: Partial<BreadcrumbItem>[]
	onClickBreadcrumbItem?: (item: BreadcrumbItem) => void
}

const props = withDefaults(defineProps<BreadcrumbProps>(), {
	autoNav: () => true,
	breadcrumbItems: () => []
})

const router = useRouter()

const handleClickItem = (item: BreadcrumbItem) => props.onClickBreadcrumbItem && props.onClickBreadcrumbItem(item)

const handleClickGoHome = () => {
	if (props.autoNav) {
		router.push(HOME_URL)
	} else {
		props.onClickBreadcrumbItem &&
			props.onClickBreadcrumbItem({
				id: 'root'
			})
	}
}
</script>

<template>
	<div class="breadcrumb-wrapper">
		<el-breadcrumb separator="›">
			<span class="el-breadcrumb__item" :class="{ current: breadcrumbItems.length === 0 }" @click="handleClickGoHome">
				<span class="el-breadcrumb__inner is-link" role="link">
					<div class="breadcrumb-item-content">文件</div>
				</span>
				<span v-if="breadcrumbItems.length > 0" class="el-breadcrumb__separator" role="presentation">›</span>
			</span>
			<span class="el-breadcrumb breadcrumb-items-wrapper">
				<template v-for="item in breadcrumbItems" :key="item.id">
					<el-breadcrumb-item
						v-if="!item.isOmit && !item.isHighlight"
						:to="{ path: autoNav ? '/home/' + item.id : '' }"
						@click="handleClickItem(item)"
					>
						<div class="breadcrumb-item-content">{{ item.text }}</div>
					</el-breadcrumb-item>
					<el-breadcrumb-item v-else-if="item.isOmit" @click="handleClickItem(item)">
						<el-icon :size="12">
							<MoreFilled />
						</el-icon>
					</el-breadcrumb-item>
					<div
						v-else-if="item.isHighlight"
						class="breadcrumb-item-content highlight"
						:to="{ path: autoNav ? '/home/' + item.id : '' }"
						@click="handleClickItem(item)"
					>
						{{ item.text }}
					</div>
				</template>
			</span>
		</el-breadcrumb>
	</div>
</template>

<style scoped lang="scss">
@use './index';
</style>
