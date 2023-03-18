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
					<el-breadcrumb-item v-if="!item.isOmit && !item.isHighlight" :to="{ path: '/home/' + item.id }">
						<div class="breadcrumb-item-content">{{ item.text }}</div>
					</el-breadcrumb-item>
					<el-breadcrumb-item v-else-if="item.isOmit">
						<el-icon :size="12">
							<MoreFilled />
						</el-icon>
					</el-breadcrumb-item>
					<div class="breadcrumb-item-content highlight" v-else-if="item.isHighlight" :to="{ path: '/home/' + item.id }">
						{{ item.text }}
					</div>
				</template>
			</span>
		</el-breadcrumb>
	</div>
</template>

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
	breadcrumbItems?: Partial<BreadcrumbItem>[]
}

withDefaults(defineProps<BreadcrumbProps>(), {
	breadcrumbItems: () => []
})

const router = useRouter()

const handleClickGoHome = () => router.push(HOME_URL)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
