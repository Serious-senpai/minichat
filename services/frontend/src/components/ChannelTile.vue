<script setup lang="ts">
import { RouterLink } from "vue-router";

import { Channel } from "../api/channels";

defineProps({ channel: Channel });
</script>

<template>
  <RouterLink class="bar d-block p-1 rounded-2 text-decoration-none text-end text-white w-100" :to="`/channels/${channel.id}`">
    {{ channel.name }}
  </RouterLink>
</template>

<style lang="css" scoped>
.bar:hover {
  background-color: gray;
}
</style>
