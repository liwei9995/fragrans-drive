<script setup lang="ts" name="login">
import LoginForm from './widgets/LoginForm.vue'
</script>

<template>
	<div class="login">
		<div class="login-wrapper">
			<div class="logo-wrapper">
				<div class="logo"></div>
				<div class="name">fragrans</div>
			</div>
			<LoginForm></LoginForm>
		</div>
	</div>
</template>

<style scoped lang="scss">
@use './index';
</style>
