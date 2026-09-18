<script setup lang="ts" name="LoginForm">
import { Key, Lock, Message, Refresh, User } from '@element-plus/icons-vue'
import type { ElForm } from 'element-plus'
import { ElMessage } from 'element-plus'
import { onMounted, onUnmounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { Auth, Login } from '@/api/interface'
import {
  authLogin,
  getAuthConfig,
  getCaptcha,
  registerUser,
  resetPassword,
  sendEmailCode,
} from '@/api/modules/user'
import { HOME_URL } from '@/config/config'
import { GlobalStore } from '@/store'

const globalStore = GlobalStore()
const router = useRouter()
const route = useRoute()

type FormInstance = InstanceType<typeof ElForm>
type Mode = 'login' | 'register' | 'forgot'

const mode = ref<Mode>('login')
const loading = ref<boolean>(false)
const sendingCode = ref<boolean>(false)
const countdown = ref<number>(0)
let timer: ReturnType<typeof setInterval> | null = null

// Auth configuration
const authConfig = ref<Auth.AuthConfig>({
  allowRegistration: true,
  emailVerificationRequired: true,
  captchaRequired: true,
})

// Graphic captcha state
const captchaId = ref<string>('')
const captchaSvg = ref<string>('')
const captchaLoading = ref<boolean>(false)

// Refresh graphic captcha
const fetchCaptcha = async () => {
  if (typeof getCaptcha !== 'function') return
  captchaLoading.value = true
  try {
    const res = await getCaptcha()
    if (res && res.id) {
      captchaId.value = res.id
      captchaSvg.value = res.svg
    }
  } catch {
    // ignore
  } finally {
    captchaLoading.value = false
  }
}

// Fetch auth configuration
const fetchConfig = async () => {
  if (typeof getAuthConfig !== 'function') return
  try {
    const res = await getAuthConfig()
    if (res) {
      authConfig.value = res
    }
  } catch {
    // ignore
  }
}

// Login form
const loginFormRef = ref<FormInstance>()
const loginForm = reactive<Login.ReqLoginForm>({
  email: '',
  password: '',
})
const loginRules = reactive({
  email: [
    { required: true, message: 'Please enter your email', trigger: 'blur' },
    {
      type: 'email',
      message: 'Please enter a valid email address',
      trigger: 'blur',
    },
  ],
  password: [
    { required: true, message: 'Please enter your password', trigger: 'blur' },
  ],
})

// Registration form
const registerFormRef = ref<FormInstance>()
const registerForm = reactive({
  firstName: '',
  lastName: '',
  email: '',
  password: '',
  confirmPassword: '',
  captchaCode: '',
  emailCode: '',
})
const registerRules = reactive({
  lastName: [
    { required: true, message: 'Please enter your last name', trigger: 'blur' },
  ],
  firstName: [
    {
      required: true,
      message: 'Please enter your first name',
      trigger: 'blur',
    },
  ],
  email: [
    { required: true, message: 'Please enter your email', trigger: 'blur' },
    {
      type: 'email',
      message: 'Please enter a valid email address',
      trigger: 'blur',
    },
  ],
  password: [
    { required: true, message: 'Please enter your password', trigger: 'blur' },
    {
      min: 6,
      message: 'Password must be at least 6 characters',
      trigger: 'blur',
    },
  ],
  confirmPassword: [
    {
      required: true,
      message: 'Please confirm your password',
      trigger: 'blur',
    },
    {
      validator: (
        _rule: unknown,
        value: string,
        callback: (error?: Error) => void,
      ) => {
        if (value !== registerForm.password) {
          callback(new Error('Passwords do not match'))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  captchaCode: [
    {
      required: true,
      message: 'Please enter the captcha code',
      trigger: 'blur',
    },
  ],
  emailCode: [
    {
      required: true,
      message: 'Please enter the verification code',
      trigger: 'blur',
    },
  ],
})

// Forgot password form
const forgotFormRef = ref<FormInstance>()
const forgotForm = reactive({
  email: '',
  captchaCode: '',
  emailCode: '',
  password: '',
  confirmPassword: '',
})
const forgotRules = reactive({
  email: [
    { required: true, message: 'Please enter your email', trigger: 'blur' },
    {
      type: 'email',
      message: 'Please enter a valid email address',
      trigger: 'blur',
    },
  ],
  captchaCode: [
    {
      required: true,
      message: 'Please enter the captcha code',
      trigger: 'blur',
    },
  ],
  emailCode: [
    {
      required: true,
      message: 'Please enter the verification code',
      trigger: 'blur',
    },
  ],
  password: [
    {
      required: true,
      message: 'Please enter your new password',
      trigger: 'blur',
    },
    {
      min: 6,
      message: 'Password must be at least 6 characters',
      trigger: 'blur',
    },
  ],
  confirmPassword: [
    {
      required: true,
      message: 'Please confirm your new password',
      trigger: 'blur',
    },
    {
      validator: (
        _rule: unknown,
        value: string,
        callback: (error?: Error) => void,
      ) => {
        if (value !== forgotForm.password) {
          callback(new Error('Passwords do not match'))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
})

// Countdown timer
const startCountdown = () => {
  countdown.value = 60
  if (timer) clearInterval(timer)
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      if (timer) clearInterval(timer)
      timer = null
    }
  }, 1000)
}

// Send email verification code
const handleSendCode = async (purpose: 'register' | 'reset_password') => {
  const targetEmail =
    purpose === 'register' ? registerForm.email : forgotForm.email
  const targetCaptcha =
    purpose === 'register' ? registerForm.captchaCode : forgotForm.captchaCode

  if (!targetEmail || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(targetEmail)) {
    ElMessage.warning('Please enter a valid email address')
    return
  }
  if (!targetCaptcha) {
    ElMessage.warning('Please enter the captcha code first')
    return
  }

  sendingCode.value = true
  try {
    const res = await sendEmailCode({
      email: targetEmail.trim(),
      purpose,
      captchaId: captchaId.value,
      captchaCode: targetCaptcha.trim(),
    })
    ElMessage.success(
      res?.message || 'Verification code has been sent to your email',
    )
    startCountdown()
  } catch {
    fetchCaptcha()
  } finally {
    sendingCode.value = false
  }
}

// Switch mode
const switchMode = (target: Mode) => {
  mode.value = target
  fetchCaptcha()
}

// Login
const login = (formEl: FormInstance | undefined) => {
  if (!formEl) return
  formEl.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const res = await authLogin({
        email: loginForm.email.trim(),
        password: loginForm.password,
      })
      const { redirect } = route.query
      const path = (redirect || HOME_URL) as string

      globalStore.setTokens(res.access_token, res.refresh_token)
      router.push(path)
    } finally {
      loading.value = false
    }
  })
}

// Register
const register = (formEl: FormInstance | undefined) => {
  if (!formEl) return
  formEl.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await registerUser({
        firstName: registerForm.firstName.trim(),
        lastName: registerForm.lastName.trim(),
        email: registerForm.email.trim(),
        password: registerForm.password,
        captchaId: captchaId.value || undefined,
        captchaCode: registerForm.captchaCode || undefined,
        emailCode: registerForm.emailCode || undefined,
      })
      ElMessage.success('Registration successful! Please sign in.')
      loginForm.email = registerForm.email.trim()
      loginForm.password = registerForm.password
      switchMode('login')
    } catch {
      fetchCaptcha()
    } finally {
      loading.value = false
    }
  })
}

// Reset password
const reset = (formEl: FormInstance | undefined) => {
  if (!formEl) return
  formEl.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await resetPassword({
        email: forgotForm.email.trim(),
        code: forgotForm.emailCode.trim(),
        password: forgotForm.password,
        changePassword: forgotForm.confirmPassword,
      })
      ElMessage.success(
        'Password reset successful! Please sign in with your new password.',
      )
      loginForm.email = forgotForm.email.trim()
      loginForm.password = ''
      switchMode('login')
    } catch {
      fetchCaptcha()
    } finally {
      loading.value = false
    }
  })
}

onMounted(() => {
  fetchConfig()
  fetchCaptcha()
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})

defineExpose({
  loginRules,
  loginFormRef,
  loginForm,
  loading,
  login,
  mode,
  registerForm,
  forgotForm,
  switchMode,
})
</script>

<template>
  <div class="login-form-wrapper">
    <!-- Log in view -->
    <div v-if="mode === 'login'" class="login-form">
      <h1>Log in</h1>
      <small>Sign in if you already have an account.</small>
      <el-form
        ref="loginFormRef"
        :model="loginForm"
        :rules="loginRules"
        size="large"
        @submit.prevent="login(loginFormRef)"
      >
        <el-form-item prop="email">
          <el-input v-model="loginForm.email" placeholder="Email">
            <template #prefix>
              <el-icon class="el-input__icon"><User /></el-icon>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item prop="password">
          <el-input
            v-model="loginForm.password"
            type="password"
            placeholder="Password"
            show-password
            autocomplete="current-password"
          >
            <template #prefix>
              <el-icon class="el-input__icon"><Lock /></el-icon>
            </template>
          </el-input>
        </el-form-item>
      </el-form>

      <div class="login-btn">
        <el-button
          class="login"
          round
          size="large"
          type="primary"
          native-type="submit"
          :disabled="loading"
          :loading="loading"
          @click="login(loginFormRef)"
        >
          Sign in
        </el-button>
      </div>

      <div class="form-links">
        <a class="link" @click.prevent="switchMode('forgot')">Forgot password?</a>
        <div v-if="authConfig.allowRegistration" class="signup-prompt">
          <span class="prompt-text">Don't have an account?</span>
          <a class="link link-primary" @click.prevent="switchMode('register')">Sign up</a>
        </div>
      </div>
    </div>

    <!-- Sign up view -->
    <div v-else-if="mode === 'register'" class="login-form">
      <h1>Sign up</h1>
      <small>Create an account to manage your files.</small>

      <!-- Notice when registration is closed -->
      <div v-if="!authConfig.allowRegistration" class="disabled-notice">
        <el-alert
          title="Registration is currently closed"
          type="warning"
          description="Public registration has been disabled by the administrator. Please contact your admin for an account."
          show-icon
          :closable="false"
        />
        <div class="login-btn" style="margin-top: 24px">
          <el-button
            class="login"
            round
            size="large"
            type="primary"
            @click="switchMode('login')"
          >
            Back to sign in
          </el-button>
        </div>
      </div>

      <!-- Registration form when open -->
      <template v-else>
        <el-form
          ref="registerFormRef"
          :model="registerForm"
          :rules="registerRules"
          size="large"
          @submit.prevent="register(registerFormRef)"
        >
          <div class="name-row">
            <el-form-item prop="lastName" class="half-item">
              <el-input v-model="registerForm.lastName" placeholder="Last name">
                <template #prefix>
                  <el-icon class="el-input__icon"><User /></el-icon>
                </template>
              </el-input>
            </el-form-item>
            <el-form-item prop="firstName" class="half-item">
              <el-input v-model="registerForm.firstName" placeholder="First name" />
            </el-form-item>
          </div>

          <el-form-item prop="email">
            <el-input v-model="registerForm.email" placeholder="Email">
              <template #prefix>
                <el-icon class="el-input__icon"><Message /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item prop="password">
            <el-input
              v-model="registerForm.password"
              type="password"
              placeholder="Password (min. 6 characters)"
              show-password
              autocomplete="new-password"
            >
              <template #prefix>
                <el-icon class="el-input__icon"><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item prop="confirmPassword">
            <el-input
              v-model="registerForm.confirmPassword"
              type="password"
              placeholder="Confirm password"
              show-password
              autocomplete="new-password"
            >
              <template #prefix>
                <el-icon class="el-input__icon"><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <!-- Graphic Captcha -->
          <el-form-item prop="captchaCode">
            <div class="captcha-row">
              <el-input
                v-model="registerForm.captchaCode"
                placeholder="Captcha code"
                maxlength="4"
              >
                <template #prefix>
                  <el-icon class="el-input__icon"><Key /></el-icon>
                </template>
              </el-input>
              <div
                class="captcha-svg-box"
                title="Click to refresh captcha"
                @click="fetchCaptcha"
              >
                <span v-if="captchaSvg" v-html="captchaSvg"></span>
                <span v-else class="captcha-placeholder">
                  <el-icon :class="{ 'is-loading': captchaLoading }"><Refresh /></el-icon>
                </span>
              </div>
            </div>
          </el-form-item>

          <!-- Email verification code (when required) -->
          <el-form-item v-if="authConfig.emailVerificationRequired" prop="emailCode">
            <div class="code-row">
              <el-input
                v-model="registerForm.emailCode"
                placeholder="6-digit verification code"
                maxlength="6"
              >
                <template #prefix>
                  <el-icon class="el-input__icon"><Key /></el-icon>
                </template>
              </el-input>
              <el-button
                class="send-btn"
                :disabled="countdown > 0 || sendingCode"
                :loading="sendingCode"
                @click="handleSendCode('register')"
              >
                {{ countdown > 0 ? `Resend in ${countdown}s` : 'Send code' }}
              </el-button>
            </div>
          </el-form-item>
        </el-form>

        <div class="login-btn">
          <el-button
            class="login"
            round
            size="large"
            type="primary"
            native-type="submit"
            :disabled="loading"
            :loading="loading"
            @click="register(registerFormRef)"
          >
            Create account
          </el-button>
        </div>

        <div class="form-links center">
          <a class="link link-primary" @click.prevent="switchMode('login')">
            Already have an account? Sign in
          </a>
        </div>
      </template>
    </div>

    <!-- Reset password view -->
    <div v-else-if="mode === 'forgot'" class="login-form">
      <h1>Reset password</h1>
      <small>Verify your email to set a new password.</small>
      <el-form
        ref="forgotFormRef"
        :model="forgotForm"
        :rules="forgotRules"
        size="large"
        @submit.prevent="reset(forgotFormRef)"
      >
        <el-form-item prop="email">
          <el-input v-model="forgotForm.email" placeholder="Account email">
            <template #prefix>
              <el-icon class="el-input__icon"><Message /></el-icon>
            </template>
          </el-input>
        </el-form-item>

        <!-- Graphic Captcha -->
        <el-form-item prop="captchaCode">
          <div class="captcha-row">
            <el-input
              v-model="forgotForm.captchaCode"
              placeholder="Captcha code"
              maxlength="4"
            >
              <template #prefix>
                <el-icon class="el-input__icon"><Key /></el-icon>
              </template>
            </el-input>
            <div
              class="captcha-svg-box"
              title="Click to refresh captcha"
              @click="fetchCaptcha"
            >
              <span v-if="captchaSvg" v-html="captchaSvg"></span>
              <span v-else class="captcha-placeholder">
                <el-icon :class="{ 'is-loading': captchaLoading }"><Refresh /></el-icon>
              </span>
            </div>
          </div>
        </el-form-item>

        <!-- Email verification code -->
        <el-form-item prop="emailCode">
          <div class="code-row">
            <el-input
              v-model="forgotForm.emailCode"
              placeholder="6-digit verification code"
              maxlength="6"
            >
              <template #prefix>
                <el-icon class="el-input__icon"><Key /></el-icon>
              </template>
            </el-input>
            <el-button
              class="send-btn"
              :disabled="countdown > 0 || sendingCode"
              :loading="sendingCode"
              @click="handleSendCode('reset_password')"
            >
              {{ countdown > 0 ? `Resend in ${countdown}s` : 'Send code' }}
            </el-button>
          </div>
        </el-form-item>

        <el-form-item prop="password">
          <el-input
            v-model="forgotForm.password"
            type="password"
            placeholder="New password (min. 6 characters)"
            show-password
            autocomplete="new-password"
          >
            <template #prefix>
              <el-icon class="el-input__icon"><Lock /></el-icon>
            </template>
          </el-input>
        </el-form-item>

        <el-form-item prop="confirmPassword">
          <el-input
            v-model="forgotForm.confirmPassword"
            type="password"
            placeholder="Confirm new password"
            show-password
            autocomplete="new-password"
          >
            <template #prefix>
              <el-icon class="el-input__icon"><Lock /></el-icon>
            </template>
          </el-input>
        </el-form-item>
      </el-form>

      <div class="login-btn">
        <el-button
          class="login"
          round
          size="large"
          type="primary"
          native-type="submit"
          :disabled="loading"
          :loading="loading"
          @click="reset(forgotFormRef)"
        >
          Reset password
        </el-button>
      </div>

      <div class="form-links center">
        <a class="link" @click.prevent="switchMode('login')">
          Remember your password? Sign in
        </a>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.login-form-wrapper {
  padding: 40px 32px;
  background: #fff;
  border-radius: 21px;
  box-shadow:
    inset 0 0 0 var(--border-width, 0) var(--border-color, transparent),
    0 var(--shadow-y, 30px) var(--shadow-blur, 130px) 0 var(--shadow, var(--c-shadow));
  max-width: 440px;
  width: 100%;
  box-sizing: border-box;

  .login-form {
    overflow: hidden;

    h1 {
      margin: 0 0 8px;
      font-weight: 600;
      color: #1e293b;
      text-align: left;
    }

    small {
      display: block;
      margin-bottom: 24px;
      font-size: 14px;
      color: #64748b;
    }

    .name-row {
      display: flex;
      gap: 12px;

      .half-item {
        flex: 1;
      }
    }

    .captcha-row {
      display: flex;
      gap: 12px;
      width: 100%;
      align-items: center;

      .captcha-svg-box {
        height: 40px;
        min-width: 120px;
        background: #f1f5f9;
        border-radius: 6px;
        border: 1px solid #e2e8f0;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        user-select: none;
        transition: opacity 0.2s;

        &:hover {
          opacity: 0.85;
        }

        :deep(svg) {
          display: block;
          height: 38px;
          width: 120px;
        }

        .captcha-placeholder {
          font-size: 18px;
          color: #94a3b8;
        }
      }
    }

    .code-row {
      display: flex;
      gap: 12px;
      width: 100%;

      .send-btn {
        white-space: nowrap;
        height: 40px;
        font-size: 13px;
        padding: 0 16px;
      }
    }

    .login-btn {
      margin-top: 8px;

      .login {
        width: 100%;
      }
    }

    .form-links {
      margin-top: 22px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 13px;
      gap: 12px;
      flex-wrap: nowrap;

      &.center {
        justify-content: center;
      }

      .link {
        color: #64748b;
        cursor: pointer;
        text-decoration: none;
        white-space: nowrap;
        transition: color 0.2s;

        &:hover {
          color: var(--el-color-primary, #008ffd);
        }

        &.link-primary {
          color: var(--el-color-primary, #008ffd);
          font-weight: 500;
          margin-left: 4px;
        }
      }

      .signup-prompt {
        display: inline-flex;
        align-items: center;
        white-space: nowrap;

        .prompt-text {
          color: #64748b;
          white-space: nowrap;
        }
      }
    }

    .disabled-notice {
      padding: 10px 0;
    }
  }
}

@media (width <= 767px) {
  .login-form-wrapper {
    padding: 24px 20px;

    .login-form {
      h1 {
        font-size: 20px;
        line-height: 28px;
      }

      small {
        margin-bottom: 16px;
      }
    }
  }
}

@media (width >= 768px) {
  .login-form-wrapper {
    .login-form {
      h1 {
        font-size: 28px;
        line-height: 28px;
      }
    }
  }
}
</style>
