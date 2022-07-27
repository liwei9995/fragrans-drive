<template>
	<div class="login-form-wrapper">
		<div class="login-form">
			<h1>Log in</h1>
			<small>Sign in if you already have an account.</small>
			<span>
				<el-form ref="loginFormRef" :model="loginForm" :rules="loginRules" size="large">
					<el-form-item prop="username">
						<el-input v-model="loginForm.username" placeholder="Email">
							<template #prefix>
								<el-icon class="el-input__icon"><user /></el-icon>
							</template>
						</el-input>
					</el-form-item>
					<el-form-item prop="password">
						<el-input
							type="password"
							v-model="loginForm.password"
							placeholder="Password"
							show-password
							autocomplete="new-password"
						>
							<template #prefix>
								<el-icon class="el-input__icon"><lock /></el-icon>
							</template>
						</el-input>
					</el-form-item>
				</el-form>
				<div class="login-btn">
					<el-button class="login" round @click="login(loginFormRef)" size="large" type="primary" :loading="loading">
						Sign in
					</el-button>
				</div>
			</span>
		</div>
	</div>
</template>

<script setup lang="ts" name="login">
import { ref, reactive, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Login } from '@/api/interface'
import type { ElForm } from 'element-plus'
import { authLogin } from '@/api/modules/user'
import { GlobalStore } from '@/store'

const globalStore = GlobalStore()

// 定义 formRef（校验规则）
type FormInstance = InstanceType<typeof ElForm>
const loginFormRef = ref<FormInstance>()
const loginRules = reactive({
	username: [
		{
			required: true,
			message: '请输入用户名',
			trigger: 'blur'
		}
	],
	password: [
		{
			required: true,
			message: '请输入密码',
			trigger: 'blur'
		}
	]
})

// 登录表单数据
const loginForm = reactive<Login.ReqLoginForm>({
	username: '',
	password: ''
})
const loading = ref<boolean>(false)
const router = useRouter()
const route = useRoute()

// login
const login = (formEl: FormInstance | undefined) => {
	if (!formEl) return
	formEl.validate(async valid => {
		if (!valid) return
		loading.value = true
		try {
			const requestLoginForm: Login.ReqLoginForm = {
				username: loginForm.username,
				password: loginForm.password
			}
			const res = await authLogin(requestLoginForm)
			const { redirect } = route.query
			const path = (redirect || '/home') as string

			// * 存储 token
			globalStore.setToken(res?.access_token)
			router.push(path)
		} finally {
			loading.value = false
		}
	})
}

onMounted(() => {
	// 监听enter事件（调用登录）
	document.onkeydown = (e: any) => {
		e = window.event || e
		if (e.code === 'Enter' || e.code === 'enter' || e.code === 'NumpadEnter') {
			if (loading.value) return
			login(loginFormRef.value)
		}
	}
})
</script>

<style scoped lang="scss">
.login-form-wrapper {
	padding: 48px 44px;
	background: #ffffff;
	border-radius: 21px;
	box-shadow: inset 0 0 0 var(--border-width, 0) var(--border-color, transparent),
		0 var(--shadow-y, 30px) var(--shadow-blur, 130px) 0 var(--shadow, var(--c-shadow));
	.login-form {
		overflow: hidden;
		h1 {
			margin: 0 0 8px;
			font-weight: 500;
			color: #27272b;
			text-align: left;
		}
		small {
			display: block;
			margin-bottom: 24px;
			font-size: 14px;
		}
		.login-btn {
			.login {
				width: 100%;
			}
		}
	}
}

@media (max-width: 767px) {
	.login-form-wrapper {
		padding: 24px;
		.login-form {
			h1 {
				font-size: 18px;
				line-height: 28px;
			}
			small {
				margin-bottom: 16px;
			}
		}
	}
}
</style>
