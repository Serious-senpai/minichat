<script setup lang="ts">
import { useRoute } from "vue-router";

import ChannelListBar from "../components/ChannelListBar.vue";

const route = useRoute();
console.log(route.params.id);
</script>

<template>
    <ChannelListBar>
    </ChannelListBar>
</template>
