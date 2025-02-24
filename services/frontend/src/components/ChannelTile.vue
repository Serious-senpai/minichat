<script setup lang="ts">
import { RouterLink } from "vue-router";

defineProps({
  id: Number,
  name: String,
});
</script>

<template>
  <RouterLink class="bar d-block p-1 rounded-2 text-decoration-none text-end text-white w-100" :to="`/channels/${id}`">
    {{ name }}
  </RouterLink>
</template>

<style lang="css" scoped>
.bar:hover {
  background-color: gray;
}
</style>
