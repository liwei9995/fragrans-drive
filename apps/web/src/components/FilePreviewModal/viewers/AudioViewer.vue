<script setup lang="ts">
import {
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
  sizeText?: string
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  name: '',
  sizeText: '',
})

const audioRef = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const volume = ref(0.8)
const isMuted = ref(false)
const isLoop = ref(false)
const isLoading = ref(true)

const formatTime = (secs: number) => {
  if (Number.isNaN(secs) || secs < 0) return '00:00'
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(m)}:${pad(s)}`
}

const progressPercent = computed(() => {
  if (!duration.value) return 0
  return (currentTime.value / duration.value) * 100
})

const togglePlay = () => {
  if (!audioRef.value) return
  if (audioRef.value.paused) {
    audioRef.value.play().catch(() => {})
  } else {
    audioRef.value.pause()
  }
}

const handleSeek = (e: MouseEvent) => {
  if (!audioRef.value || !duration.value) return
  const target = e.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const clickX = e.clientX - rect.left
  const ratio = Math.max(0, Math.min(1, clickX / rect.width))
  audioRef.value.currentTime = ratio * duration.value
}

const toggleMute = () => {
  if (!audioRef.value) return
  isMuted.value = !isMuted.value
  audioRef.value.muted = isMuted.value
}

const toggleLoop = () => {
  isLoop.value = !isLoop.value
  if (audioRef.value) audioRef.value.loop = isLoop.value
}

const handleVolumeChange = (val: number) => {
  if (!audioRef.value) return
  volume.value = val
  audioRef.value.volume = val
  if (val > 0 && isMuted.value) {
    isMuted.value = false
    audioRef.value.muted = false
  }
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!audioRef.value) return
  if ((e.target as HTMLElement)?.tagName === 'INPUT') return

  if (e.code === 'Space') {
    e.preventDefault()
    togglePlay()
  } else if (e.code === 'ArrowRight') {
    e.preventDefault()
    audioRef.value.currentTime = Math.min(
      duration.value,
      audioRef.value.currentTime + 5,
    )
  } else if (e.code === 'ArrowLeft') {
    e.preventDefault()
    audioRef.value.currentTime = Math.max(0, audioRef.value.currentTime - 5)
  }
}

const onTimeUpdate = () => {
  if (audioRef.value) currentTime.value = audioRef.value.currentTime
}

const onLoadedMetadata = () => {
  if (audioRef.value) {
    duration.value = audioRef.value.duration
    isLoading.value = false
  }
}

const onPlay = () => {
  isPlaying.value = true
}

const onPause = () => {
  isPlaying.value = false
}

const onEnded = () => {
  if (!isLoop.value) {
    isPlaying.value = false
    currentTime.value = 0
  }
}

watch(
  () => props.src,
  () => {
    isLoading.value = true
    currentTime.value = 0
    duration.value = 0
  },
)

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (audioRef.value) {
    audioRef.value.pause()
  }
})
</script>

<template>
  <div class="audio-viewer">
    <audio
      ref="audioRef"
      :src="src"
      :loop="isLoop"
      @timeupdate="onTimeUpdate"
      @loadedmetadata="onLoadedMetadata"
      @play="onPlay"
      @pause="onPause"
      @ended="onEnded"
    />

    <div class="audio-card">
      <!-- Vinyl Disc Artwork -->
      <div class="vinyl-container">
        <div class="vinyl-disc" :class="{ playing: isPlaying }">
          <div class="groove groove-1"></div>
          <div class="groove groove-2"></div>
          <div class="groove groove-3"></div>
          <div class="vinyl-center">
            <span class="music-note">🎵</span>
          </div>
        </div>
      </div>

      <!-- Track Info -->
      <div class="track-info">
        <h3 class="track-title" :title="name">{{ name || '音频播放' }}</h3>
        <p v-if="sizeText" class="track-meta">{{ sizeText }} · 优质无损音频</p>
      </div>

      <!-- Animated Waveform Visualizer Bars -->
      <div class="waveform-bars" :class="{ active: isPlaying }">
        <span v-for="i in 28" :key="i" class="bar" :style="{ '--delay': `${(i % 5) * 0.15}s` }" />
      </div>

      <!-- Scrubber -->
      <div class="time-scrubber" @click="handleSeek">
        <div class="scrub-track">
          <div class="scrub-progress" :style="{ width: `${progressPercent}%` }">
            <div class="scrub-thumb"></div>
          </div>
        </div>
        <div class="time-labels">
          <span>{{ formatTime(currentTime) }}</span>
          <span>{{ formatTime(duration) }}</span>
        </div>
      </div>

      <!-- Player Action Controls -->
      <div class="player-controls">
        <button
          class="ctrl-btn loop-btn"
          :class="{ active: isLoop }"
          :title="isLoop ? '单曲循环中' : '列表顺序播放'"
          @click="toggleLoop"
        >
          <span class="btn-icon">🔁</span>
        </button>

        <button class="play-main-btn" :title="isPlaying ? '暂停 (Space)' : '播放 (Space)'" @click="togglePlay">
          <el-icon v-if="isLoading" class="is-loading" :size="24"><Refresh /></el-icon>
          <el-icon v-else-if="isPlaying" :size="26"><VideoPause /></el-icon>
          <el-icon v-else :size="26"><VideoPlay /></el-icon>
        </button>

        <!-- Volume Popover/Slider -->
        <div class="volume-container">
          <button class="ctrl-btn" :title="isMuted ? '取消静音' : '静音'" @click="toggleMute">
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
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.audio-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: radial-gradient(circle at 50% 40%, #1e1b4b 0%, #0f172a 100%);
  user-select: none;
  padding: 24px;

  .audio-card {
    width: 100%;
    max-width: 440px;
    background: rgba(30, 41, 59, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border-radius: 24px;
    padding: 36px 32px 28px;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.6);

    .vinyl-container {
      position: relative;
      margin-bottom: 24px;

      .vinyl-disc {
        width: 180px;
        height: 180px;
        border-radius: 50%;
        background: radial-gradient(circle, #2d3748 0%, #1a202c 40%, #111827 70%, #030712 100%);
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6), inset 0 0 0 2px rgba(255, 255, 255, 0.05);
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
        animation: spin 8s linear infinite paused;

        &.playing {
          animation-play-state: running;
        }

        .groove {
          position: absolute;
          border-radius: 50%;
          border: 1px dashed rgba(255, 255, 255, 0.06);

          &-1 { width: 140px; height: 140px; }
          &-2 { width: 110px; height: 110px; }
          &-3 { width: 80px; height: 80px; }
        }

        .vinyl-center {
          width: 54px;
          height: 54px;
          border-radius: 50%;
          background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
          display: flex;
          align-items: center;
          justify-content: center;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);

          .music-note {
            font-size: 20px;
          }
        }
      }
    }

    .track-info {
      text-align: center;
      margin-bottom: 20px;
      width: 100%;

      .track-title {
        font-size: 18px;
        font-weight: 600;
        color: #f8fafc;
        margin: 0 0 6px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .track-meta {
        font-size: 13px;
        color: #94a3b8;
        margin: 0;
      }
    }

    .waveform-bars {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      height: 28px;
      margin-bottom: 24px;
      width: 100%;

      .bar {
        width: 3px;
        height: 6px;
        border-radius: 2px;
        background: rgba(148, 163, 184, 0.4);
        transition: height 0.2s ease, background 0.2s ease;
      }

      &.active .bar {
        background: #ec4899;
        animation: pulse 1s ease-in-out infinite alternate var(--delay);
      }
    }

    .time-scrubber {
      width: 100%;
      cursor: pointer;
      margin-bottom: 20px;

      .scrub-track {
        position: relative;
        width: 100%;
        height: 5px;
        background: rgba(255, 255, 255, 0.15);
        border-radius: 3px;

        .scrub-progress {
          position: absolute;
          left: 0;
          top: 0;
          bottom: 0;
          background: linear-gradient(90deg, #ec4899, #a855f7);
          border-radius: 3px;
          display: flex;
          align-items: center;
          justify-content: flex-end;

          .scrub-thumb {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: #fff;
            box-shadow: 0 0 6px rgba(0, 0, 0, 0.4);
            transform: translateX(50%);
          }
        }
      }

      .time-labels {
        display: flex;
        justify-content: space-between;
        margin-top: 8px;
        font-size: 12px;
        color: #94a3b8;
        font-variant-numeric: tabular-nums;
      }
    }

    .player-controls {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
      padding: 0 8px;

      .ctrl-btn {
        background: transparent;
        border: none;
        color: #94a3b8;
        cursor: pointer;
        padding: 8px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;

        &:hover {
          color: #f8fafc;
          background: rgba(255, 255, 255, 0.08);
        }

        &.active {
          color: #ec4899;
        }

        .btn-icon {
          font-size: 16px;
        }
      }

      .play-main-btn {
        width: 56px;
        height: 56px;
        border-radius: 50%;
        background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
        border: none;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        box-shadow: 0 10px 20px -5px rgba(236, 72, 153, 0.5);
        transition: all 0.2s ease;

        &:hover {
          transform: scale(1.06);
          box-shadow: 0 14px 24px -5px rgba(236, 72, 153, 0.6);
        }

        &:active {
          transform: scale(0.96);
        }
      }

      .volume-container {
        display: flex;
        align-items: center;
        gap: 4px;

        .volume-slider {
          width: 60px;
          height: 4px;
          accent-color: #ec4899;
          cursor: pointer;
        }
      }
    }
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes pulse {
  0% { height: 4px; }
  100% { height: 26px; }
}

@media (prefers-color-scheme: light) {
  .audio-viewer {
    background: radial-gradient(circle at 50% 40%, rgba(248, 250, 252, 0.8) 0%, rgba(226, 232, 240, 0.9) 100%);

    .audio-card {
      background: rgba(255, 255, 255, 0.88);
      border: 1px solid rgba(0, 0, 0, 0.08);
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.12);

      .track-info {
        .track-title {
          color: #0f172a;
        }

        .track-meta {
          color: #64748b;
        }
      }

      .time-scrubber {
        .scrub-track {
          background: rgba(0, 0, 0, 0.08);
        }

        .time-labels {
          color: #64748b;
        }
      }

      .player-controls {
        .ctrl-btn {
          color: #64748b;

          &:hover {
            color: #0f172a;
            background: rgba(0, 0, 0, 0.06);
          }

          &.active {
            color: #ec4899;
          }
        }
      }
    }
  }
}
</style>
