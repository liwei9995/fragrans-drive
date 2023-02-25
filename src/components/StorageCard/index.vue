<template>
	<div class="card-wrapper" @click="handleClickCard">
		<div class="drop-wrapper">
			<div class="card-container">
				<div class="outer-wrapper" @mouseover="showMoreAction = true" @mouseleave="showMoreAction = false">
					<div class="action-btn"></div>
					<el-dropdown class="action-btn-more" :class="{ show: showMoreAction }" trigger="click" @command="handleCommand">
						<el-icon :size="18">
							<More />
						</el-icon>
						<template #dropdown>
							<el-dropdown-menu>
								<el-dropdown-item v-for="item in actionItems" :key="item.id" :divided="item.divided" :command="item.id">
									{{ item.name }}
								</el-dropdown-item>
							</el-dropdown-menu>
						</template>
					</el-dropdown>
					<div class="node-card">
						<div class="cover">
							<div class="folder-cover">
								<el-image class="file-icon" alt="folder" :src="thumbUrl" fit="cover" />
							</div>
						</div>
						<div class="info">
							<p class="title">{{ title }}</p>
							<p class="desc">{{ desc }}</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="storage-card">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { HOME_URL } from '@/config/config'

const router = useRouter()
const showMoreAction = ref(false)

type ActionItem = {
	id?: string
	name: string
	divided?: boolean
}

interface StorageCardProps {
	id: string
	title: string
	mimeType?: string
	type?: string
	desc: string
	thumbUrl?: string
	actionItems?: Partial<ActionItem>[]
	tapActionItem?: (command: string | number | object, id: string, title: string, mimeType?: string, thumbUrl?: string) => void
}

const props = withDefaults(defineProps<StorageCardProps>(), {
	title: '',
	desc: '',
	actionItems: () => [
		{
			id: 'download',
			name: '下载',
			divided: false
		},
		{
			id: 'rename',
			name: '重命名',
			divided: false
		},
		{
			id: 'delete',
			name: '删除',
			divided: true
		}
	],
	thumbUrl: 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png'
})

const handleCommand = (command: string | number | object) =>
	props.tapActionItem && props.tapActionItem(command, props.id, props.title, props.mimeType, props.thumbUrl)

const handleClickCard = () => {
	if (props.type === 'folder') {
		router.push(`${HOME_URL}/${props.id}`)
	}
}
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
