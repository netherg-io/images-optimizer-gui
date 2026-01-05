<script setup>
definePageMeta({
  key: 'fixed',
});
</script>

<template>
  <div class="pages-index">
    <SectionHead description="sections.head.description" />
  </div>
</template>

<style lang="scss" scoped>
.pages-index {
  display: flex;
  flex-direction: column;
  gap: em(32);
  padding-block: em(32);
}
</style>
