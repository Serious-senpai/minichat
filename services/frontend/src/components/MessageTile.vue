<script setup lang="ts">
defineProps({
  author: String,
  content: String,
});
</script>

<template>
  <div class="w-100">
    <div class="fs-6 text-info">
      {{ author }}
    </div>
    <div class="fs-6 text-white">
      {{ content }}
    </div>
  </div>
</template>
