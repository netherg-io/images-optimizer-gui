<template>
  <CResize id="app" class="app">
    <NuxtLayout v-slot="{ className }" class="app__layout">
      <div :class="className">
        <NuxtPage class="app__page" :keepalive="false" />
      </div>
    </NuxtLayout>
  </CResize>
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window';

useTheme();

onMounted(async () => {
  await getCurrentWindow().show();

  await getCurrentWindow().setFocus();
});
</script>

<style scoped lang="scss">
.app {
  $parent: &;

  display: flex;
  flex-direction: column;

  &:deep(#{$parent}__layout) {
    flex-grow: 1;
  }

  &:deep(#{$parent}__page) {
    flex-grow: 1;
    width: 100%;
  }
}
</style>
