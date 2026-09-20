<script setup lang="ts">
import {
  FullScreen,
  Microphone,
  Mute,
  Refresh,
  VideoPause,
  VideoPlay,
} from '@element-plus/icons-vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface Props {
  src: string
  name?: string
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
})

const videoRef = ref<HTMLVideoElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)

const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const bufferedPercent = ref(0)
const volume = ref(1)
const isMuted = ref(false)
const playbackRate = ref(1)
const isFullscreen = ref(false)
const isLoading = ref(true)
const showControls = ref(true)
let hideControlsTimer: ReturnType<typeof setTimeout> | null = null

const formatTime = (secs: number) => {
  if (Number.isNaN(secs) || secs < 0) return '00:00'
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  const pad = (n: number) => String(n).padStart(2, '0')
  if (m >= 60) {
    const h = Math.floor(m / 60)
    return `${pad(h)}:${pad(m % 60)}:${pad(s)}`
  }
  return `${pad(m)}:${pad(s)}`
}

const progressPercent = computed(() => {
  if (!duration.value) return 0
  return (currentTime.value / duration.value) * 100
})

const togglePlay = () => {
  if (!videoRef.value) return
  if (videoRef.value.paused) {
    videoRef.value.play().catch(() => {})
  } else {
    videoRef.value.pause()
  }
}

const handleSeek = (e: MouseEvent) => {
  if (!videoRef.value || !duration.value) return
  const target = e.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const clickX = e.clientX - rect.left
  const ratio = Math.max(0, Math.min(1, clickX / rect.width))
  videoRef.value.currentTime = ratio * duration.value
}

const toggleMute = () => {
  if (!videoRef.value) return
  isMuted.value = !isMuted.value
  videoRef.value.muted = isMuted.value
}

const handleVolumeChange = (val: number) => {
  if (!videoRef.value) return
  volume.value = val
  videoRef.value.volume = val
  if (val > 0 && isMuted.value) {
    isMuted.value = false
    videoRef.value.muted = false
  }
}

const changeSpeed = (rate: number) => {
  playbackRate.value = rate
  if (videoRef.value) videoRef.value.playbackRate = rate
}

const toggleFullscreen = () => {
  if (!containerRef.value) return
  if (!document.fullscreenElement) {
    containerRef.value.requestFullscreen?.().catch(() => {})
  } else {
    document.exitFullscreen?.().catch(() => {})
  }
}

const togglePiP = async () => {
  if (!videoRef.value) return
  try {
    if (document.pictureInPictureElement) {
      await document.exitPictureInPicture()
    } else if (document.pictureInPictureEnabled) {
      await videoRef.value.requestPictureInPicture()
    }
  } catch (err) {
    console.warn('PiP error:', err)
  }
}

const handleMouseMove = () => {
  showControls.value = true
  if (hideControlsTimer) clearTimeout(hideControlsTimer)
  if (isPlaying.value) {
    hideControlsTimer = setTimeout(() => {
      showControls.value = false
    }, 3000)
  }
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!videoRef.value) return
  // Don't capture when typing inside input
  if ((e.target as HTMLElement)?.tagName === 'INPUT') return

  if (e.code === 'Space') {
    e.preventDefault()
    togglePlay()
  } else if (e.code === 'ArrowRight') {
    e.preventDefault()
    videoRef.value.currentTime = Math.min(
      duration.value,
      videoRef.value.currentTime + 5,
    )
  } else if (e.code === 'ArrowLeft') {
    e.preventDefault()
    videoRef.value.currentTime = Math.max(0, videoRef.value.currentTime - 5)
  } else if (e.code === 'ArrowUp') {
    e.preventDefault()
    handleVolumeChange(Math.min(1, volume.value + 0.1))
  } else if (e.code === 'ArrowDown') {
    e.preventDefault()
    handleVolumeChange(Math.max(0, volume.value - 0.1))
  } else if (e.key === 'm' || e.key === 'M') {
    toggleMute()
  } else if (e.key === 'f' || e.key === 'F') {
    toggleFullscreen()
  }
}

const onProgress = () => {
  if (!videoRef.value || !duration.value) return
  const buf = videoRef.value.buffered
  if (buf.length > 0) {
    const cur = videoRef.value.currentTime
    let maxBuf = 0
    for (let i = 0; i < buf.length; i++) {
      if (buf.start(i) <= cur && cur <= buf.end(i)) {
        maxBuf = buf.end(i)
        break
      }
      if (buf.end(i) > maxBuf) {
        maxBuf = buf.end(i)
      }
    }
    bufferedPercent.value = Math.min(100, (maxBuf / duration.value) * 100)
  }
}

const onTimeUpdate = () => {
  if (videoRef.value) {
    currentTime.value = videoRef.value.currentTime
    onProgress()
  }
}

const onLoadedMetadata = () => {
  if (videoRef.value) {
    duration.value = videoRef.value.duration
    isLoading.value = false
    onProgress()
  }
}

const onPlay = () => {
  isPlaying.value = true
}

const onPause = () => {
  isPlaying.value = false
  showControls.value = true
}

const onWaiting = () => {
  isLoading.value = true
}

const onPlaying = () => {
  isLoading.value = false
}

const onFullscreenChange = () => {
  isFullscreen.value = !!document.fullscreenElement
}

watch(
  () => props.src,
  () => {
    isLoading.value = true
    currentTime.value = 0
    duration.value = 0
    bufferedPercent.value = 0
  },
)

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('fullscreenchange', onFullscreenChange)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('fullscreenchange', onFullscreenChange)
  if (videoRef.value) {
    videoRef.value.pause()
  }
  if (hideControlsTimer) clearTimeout(hideControlsTimer)
})
</script>

<template>
  <div
    ref="containerRef"
    class="video-viewer"
    :class="{ 'fullscreen-active': isFullscreen }"
    @mousemove="handleMouseMove"
    @mouseleave="isPlaying && (showControls = false)"
  >
    <div class="video-stage" @click="togglePlay">
      <video
        ref="videoRef"
        :src="src"
        class="video-element"
        playsinline
        preload="auto"
        @timeupdate="onTimeUpdate"
        @loadedmetadata="onLoadedMetadata"
        @progress="onProgress"
        @play="onPlay"
        @pause="onPause"
        @waiting="onWaiting"
        @playing="onPlaying"
      />

      <!-- Center loading spinner -->
      <div v-if="isLoading" class="center-spinner">
        <el-icon class="is-loading" :size="48"><Refresh /></el-icon>
      </div>

      <!-- Center play pause indicator icon -->
      <transition name="el-fade-in">
        <div v-if="!isPlaying && !isLoading" class="center-play-badge">
          <el-icon :size="48"><VideoPlay /></el-icon>
        </div>
      </transition>
    </div>

    <!-- Custom Control Bar -->
    <transition name="fade">
      <div v-show="showControls || !isPlaying" class="controls-overlay" @click.stop>
        <!-- Scrub bar -->
        <div class="scrub-container" @click="handleSeek">
          <div class="scrub-track">
            <div
              class="scrub-buffered"
              :style="{ width: `${bufferedPercent}%` }"
            />
            <div
              class="scrub-progress"
              :style="{ width: `${progressPercent}%` }"
            >
              <div class="scrub-handle"></div>
            </div>
          </div>
        </div>

        <!-- Controls Row -->
        <div class="controls-row">
          <div class="left-controls">
            <button class="ctrl-btn" :title="isPlaying ? '暂停 (Space)' : '播放 (Space)'" @click="togglePlay">
              <el-icon :size="20">
                <VideoPause v-if="isPlaying" />
                <VideoPlay v-else />
              </el-icon>
            </button>

            <div class="time-display">
              <span class="curr">{{ formatTime(currentTime) }}</span>
              <span class="sep">/</span>
              <span class="dur">{{ formatTime(duration) }}</span>
            </div>
          </div>

          <div class="right-controls">
            <!-- Speed Menu -->
            <el-dropdown trigger="click" @command="changeSpeed">
              <button class="speed-btn" title="播放倍速">
                {{ playbackRate === 1 ? '倍速' : `${playbackRate}x` }}
              </button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item
                    v-for="rate in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]"
                    :key="rate"
                    :command="rate"
                    :class="{ 'is-active': playbackRate === rate }"
                  >
                    {{ rate }}x
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>

            <!-- Volume -->
            <div class="volume-group">
              <button class="ctrl-btn" :title="isMuted ? '取消静音 (M)' : '静音 (M)'" @click="toggleMute">
                <el-icon :size="18">
                  <Mute v-if="isMuted || volume === 0" />
                  <Microphone v-else />
                </el-icon>
              </button>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                :value="isMuted ? 0 : volume"
                class="volume-slider"
                @input="(e) => handleVolumeChange(Number((e.target as HTMLInputElement).value))"
              />
            </div>

            <!-- PiP -->
            <button class="ctrl-btn" title="画中画模式" @click="togglePiP">
              <span class="pip-icon">🗔</span>
            </button>

            <!-- Fullscreen -->
            <button class="ctrl-btn" :title="isFullscreen ? '退出全屏 (F)' : '全屏 (F)'" @click="toggleFullscreen">
              <el-icon :size="18"><FullScreen /></el-icon>
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped lang="scss">
.video-viewer {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  overflow: hidden;
  user-select: none;

  .video-stage {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;

    .video-element {
      max-width: 100%;
      max-height: 100%;
      outline: none;
    }

    .center-spinner {
      position: absolute;
      color: var(--c-primary, #008ffd);
      pointer-events: none;
    }

    .center-play-badge {
      position: absolute;
      width: 72px;
      height: 72px;
      border-radius: 50%;
      background: rgba(15, 23, 42, 0.7);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255, 255, 255, 0.2);
      display: flex;
      align-items: center;
      justify-content: center;
      color: #fff;
      pointer-events: none;
      box-shadow: 0 10px 25px rgba(0, 0, 0, 0.4);
    }
  }

  .controls-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.85) 0%, rgba(0, 0, 0, 0.3) 70%, transparent 100%);
    padding: 20px 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    z-index: 10;

    .scrub-container {
      width: 100%;
      height: 16px;
      display: flex;
      align-items: center;
      cursor: pointer;

      .scrub-track {
        position: relative;
        width: 100%;
        height: 4px;
        background: rgba(255, 255, 255, 0.25);
        border-radius: 2px;
        transition: height 0.15s ease;

        .scrub-buffered {
          position: absolute;
          left: 0;
          top: 0;
          bottom: 0;
          background: rgba(255, 255, 255, 0.35);
          border-radius: 2px;
          pointer-events: none;
          transition: width 0.2s ease;
        }

        .scrub-progress {
          position: absolute;
          left: 0;
          top: 0;
          bottom: 0;
          background: var(--c-primary, #008ffd);
          border-radius: 2px;
          display: flex;
          align-items: center;
          justify-content: flex-end;

          .scrub-handle {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: #fff;
            box-shadow: 0 0 6px rgba(0, 0, 0, 0.5);
            transform: translateX(50%) scale(0);
            transition: transform 0.15s ease;
          }
        }
      }

      &:hover .scrub-track {
        height: 6px;
        .scrub-handle {
          transform: translateX(50%) scale(1);
        }
      }
    }

    .controls-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;

      @media (max-width: 600px) {
        gap: 6px;
      }

      .left-controls,
      .right-controls {
        display: flex;
        align-items: center;
        gap: 12px;

        @media (max-width: 600px) {
          gap: 6px;
        }
      }

      .ctrl-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: 6px;
        background: transparent;
        border: none;
        color: #f1f5f9;
        cursor: pointer;
        transition: all 0.2s ease;

        @media (max-width: 600px) {
          width: 28px;
          height: 28px;
        }

        &:hover {
          background: rgba(255, 255, 255, 0.15);
          color: var(--c-primary-light, #33a5fd);
        }

        .pip-icon {
          font-size: 16px;
        }
      }

      .time-display {
        font-size: 13px;
        color: #cbd5e1;
        font-variant-numeric: tabular-nums;
        display: flex;
        align-items: center;
        gap: 4px;
        margin-left: 4px;
        white-space: nowrap;

        @media (max-width: 600px) {
          font-size: 11px;
          margin-left: 0;
        }

        .sep {
          color: #64748b;
        }
      }

      .speed-btn {
        background: transparent;
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: #f1f5f9;
        font-size: 12px;
        padding: 4px 8px;
        border-radius: 4px;
        cursor: pointer;
        transition: all 0.2s;
        white-space: nowrap;

        @media (max-width: 600px) {
          padding: 2px 6px;
          font-size: 11px;
        }

        &:hover {
          border-color: var(--c-primary-light, #33a5fd);
          color: var(--c-primary-light, #33a5fd);
        }
      }

      .volume-group {
        display: flex;
        align-items: center;
        gap: 6px;

        .volume-slider {
          width: 70px;
          height: 4px;
          accent-color: var(--c-primary, #008ffd);
          cursor: pointer;

          @media (max-width: 600px) {
            display: none;
          }
        }
      }
    }
  }

  .fade-enter-active,
  .fade-leave-active {
    transition: opacity 0.25s ease;
  }
  .fade-enter-from,
  .fade-leave-to {
    opacity: 0;
  }
}
</style>
