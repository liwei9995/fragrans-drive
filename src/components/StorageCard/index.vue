<template>
	<div class="card-wrapper" :class="{ empty: isEmpty }">
		<div v-if="!isEmpty" class="drop-wrapper">
			<div class="card-container" @click="handleClickCard">
				<div class="outer-wrapper" @mouseover="showMoreAction = true" @mouseleave="showMoreAction = false">
					<div class="action-btn"></div>
					<div class="action-btn-more-wrapper" @click.stop>
						<el-dropdown
							class="action-btn-more"
							:class="{ show: showMoreAction || isMobile }"
							trigger="click"
							@command="handleCommand"
						>
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
					</div>
					<div class="node-card">
						<div class="cover">
							<div :class="type + '-cover'">
								<div class="file-icon" :class="{ thumb: previewSrcList.length > 0 }">
									<el-image
										class="icon"
										:class="{ show: !showPlaceholder }"
										alt="folder"
										:src="thumbUrl"
										:preview-src-list="previewSrcList"
										fit="contain"
										@close="handleClosePreview"
										@load="handleLoad"
									/>
									<div
										v-if="showPlaceholder"
										class="icon-placeholder"
										:style="{ backgroundImage: 'url(' + thumbPlaceholder + ')' }"
									/>
								</div>
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
import { GlobalStore } from '@/store'

const router = useRouter()
const showMoreAction = ref(false)
const showPlaceholder = ref(true)
const globalStore = GlobalStore()
const isMobile = globalStore.isMobile

type ActionItem = {
	id?: string
	name: string
	divided?: boolean
}

interface StorageCardProps {
	id: string
	title?: string
	mimeType?: string
	type?: string
	desc?: string
	previewSrcList?: string[]
	thumbUrl?: string
	thumbPlaceholder?: string
	isEmpty?: boolean
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
	previewSrcList: () => [],
	thumbUrl: 'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png',
	thumbPlaceholder: '',
	isEmpty: false
})

const handleCommand = (command: string | number | object) =>
	props.tapActionItem && props.tapActionItem(command, props.id, props.title, props.mimeType, props.thumbUrl)

const handleClickCard = () => {
	if (props.type === 'folder') {
		router.push(`${HOME_URL}/${props.id}`)
	}
}

const handleClosePreview = () => (showMoreAction.value = false)

const handleLoad = () => (showPlaceholder.value = false)
</script>

<style scoped lang="scss">
@import './index.scss';
</style>
