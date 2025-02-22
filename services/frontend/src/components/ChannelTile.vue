<script setup lang="ts">
import { RouterLink } from "vue-router";

defineProps({
  channel_id: Number,
  channel_name: String,
});
</script>

<template>
  <RouterLink class="bar d-block p-1 rounded-2 text-decoration-none text-end text-white w-100" :to="`/channels/${channel_id}`">
    {{ channel_name }}
  </RouterLink>
</template>

<style lang="css" scoped>
.bar:hover {
  background-color: gray;
}
</style>
