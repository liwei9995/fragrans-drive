<script setup lang="ts" name="storage-card">
import { computed, ref } from 'vue'
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
  extName?: string
  desc?: string
  previewSrcList?: string[]
  thumbUrl?: string
  thumbPlaceholder?: string
  videoUrl?: string
  isEmpty?: boolean
  actionItems?: Partial<ActionItem>[]
  tapActionItem?: (
    command: string | number | object,
    id: string,
    title: string,
    mimeType?: string,
    thumbUrl?: string,
    extName?: string,
  ) => void
  previewVideo?: (videoUrl: string) => void
  selected?: boolean
}

const emit = defineEmits(['toggle-select'])

const props = withDefaults(defineProps<StorageCardProps>(), {
  title: '',
  desc: '',
  extName: '',
  actionItems: () => [
    {
      id: 'download',
      name: '下载',
      divided: false,
    },
    {
      id: 'rename',
      name: '重命名',
      divided: false,
    },
    {
      id: 'move',
      name: '移动',
      divided: false,
    },
    {
      id: 'delete',
      name: '删除',
      divided: true,
    },
  ],
  previewSrcList: () => [],
  thumbUrl:
    'https://img.alicdn.com/imgextra/i1/O1CN01rGJZac1Zn37NL70IT_!!6000000003238-2-tps-230-180.png',
  thumbPlaceholder: '',
  videoUrl: '',
  isEmpty: false,
})

const coverCls = computed(() =>
  showPlaceholder.value ? 'file-cover' : `${props.type}-cover`,
)

const handleCommand = (command: string | number | object) =>
  props.tapActionItem &&
  props.tapActionItem(
    command,
    props.id,
    props.title,
    props.mimeType,
    props.thumbUrl,
    props.extName,
  )

const handleClickCard = () => {
  if (props.type === 'folder') {
    router.push(`${HOME_URL}/${props.id}`)
  }
}

const handleClickIcon = () => {
  if (props.mimeType?.startsWith('video/')) {
    props.previewVideo && props.previewVideo(props.videoUrl)
  }
}

const handleClosePreview = () => (showMoreAction.value = false)

const handleLoad = () => (showPlaceholder.value = false)
</script>

<template>
  <div class="card-wrapper" :class="{ empty: isEmpty }">
    <div v-if="!isEmpty" class="drop-wrapper">
      <el-dropdown trigger="contextmenu" @command="handleCommand" placement="bottom-start" popper-class="premium-context-menu">
        <div class="card-container" @click="handleClickCard">
          <div class="outer-wrapper" @mouseover="showMoreAction = true" @mouseleave="showMoreAction = false">
            <div class="action-btn"></div>
          <div class="selection-checkbox" :class="{ visible: selected }" @click.stop="emit('toggle-select', id)">
            <el-icon v-if="selected" class="check-icon"><SuccessFilled /></el-icon>
            <div v-else class="check-placeholder"></div>
          </div>
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
            <div class="cover" @click="handleClickIcon">
              <div :class="coverCls">
                <div class="file-icon" :class="{ thumb: !showPlaceholder && previewSrcList.length > 0 }">
                  <el-image
                    class="icon"
                    :class="{ show: !showPlaceholder }"
                    alt="source"
                    :src="thumbUrl"
                    :preview-src-list="previewSrcList"
                    :initial-index="0"
                    preview-teleported
                    hide-on-click-modal
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
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item v-for="item in actionItems" :key="item.id" :divided="item.divided" :command="item.id">
            {{ item.name }}
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
      </el-dropdown>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use './index';
</style>
