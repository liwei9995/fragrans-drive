<template>
	<div class="home flx-center">
		<div class="content">
			<div class="file-drag-zone">
				<div class="page-content">
					<header class="header">
						<div class="page-title">
							<el-breadcrumb separator="›">
								<el-breadcrumb-item>文件</el-breadcrumb-item>
								<el-breadcrumb-item>
									<el-icon :size="18"><MoreFilled /></el-icon>
								</el-breadcrumb-item>
								<el-breadcrumb-item>NBA</el-breadcrumb-item>
							</el-breadcrumb>
						</div>
						<div class="actions">
							<div class="search">
								<el-icon :size="24">
									<Search />
								</el-icon>
							</div>
							<div class="action">
								<el-icon :size="32">
									<CirclePlusFilled />
								</el-icon>
							</div>
						</div>
					</header>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts" name="home"></script>

<style scoped lang="scss">
@import './index.scss';

.home {
	height: 100%;

	.content {
		position: relative;
		box-sizing: border-box;
		flex: 1 1;
		height: 100%;
		padding-top: 28px;
		overflow: hidden;

		.file-drag-zone {
			.page-content {
				.header {
					display: flex;
					flex-shrink: 0;
					align-items: center;
					justify-content: space-between;
					height: 40px;
					padding: 0 40px;
					margin-bottom: 24px;
					white-space: nowrap;

					.page-title {
						position: relative;
						display: flex;
						flex-direction: row;
						flex-grow: 1;
						align-items: center;
						height: 100%;
						margin-right: 24px;
						overflow: hidden;

						.el-breadcrumb {
							min-width: 24px;
							font-size: 18px;
							line-height: 1.4;
						}
					}

					.actions {
						display: flex;
						flex-shrink: 0;
						height: 32px;
						color: #008ffd;

						.search {
							display: flex;
							padding: 4px;
							cursor: pointer;
							border-radius: 50%;
							transition: background-color 0.3s ease;
						}

						.action {
							height: 32px;
							margin-left: 20px;
						}
					}
				}
			}
		}
	}
}
</style>
