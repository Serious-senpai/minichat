<script setup lang="ts">
import { Message } from "../api/channels";

defineProps({ message: Message });
</script>

<template>
  <div class="w-100">
    <div>
      <span class="fs-6 text-info">{{ message.author.username }}</span>
      <span class="time text-secondary">{{ message.createdAt }}</span>
    </div>
    <div class="fs-6 text-white">
      {{ message.content }}
    </div>
  </div>
</template>

<style lang="css" scoped>
.time {
  font-size: 0.7rem;
  margin-left: 1rem;
}
</style>
