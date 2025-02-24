<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useRoute } from "vue-router";
import type { Ref } from "vue";

import { Channel, Message } from "../api/channels";
import ChannelListBar from "../components/ChannelListBar.vue";
import MessageTile from "../components/MessageTile.vue";

const input = ref("");

const history: Ref<Message[]> = ref([]);
const historyContainer = useTemplateRef("channel-history");

let channel: Channel | null = null;
let loading = false;
let oldestMessageId = Number.MAX_SAFE_INTEGER;
let noMoreMessages = false;

async function fetchHistory(): Promise<void> {
  if (channel) {
    const older = await channel.history({
      newest: true,
      beforeId: oldestMessageId - 1,
      afterId: 0,
      limit: 50,
    });

    if (older.length === 0) {
      noMoreMessages = true;
    }

    history.value.push(...older);
    oldestMessageId = older[older.length - 1].id;
  }
}

async function scrollHistory(): Promise<void> {
  const element = historyContainer.value;
  if (element) {
    const limit = element.clientHeight - element.scrollHeight; // negative value, since we're scrolling up
    if (element.scrollTop - limit < 10 && !loading && !noMoreMessages) {
      loading = true;
      await fetchHistory();
      loading = false;
    }
  }
}

function send(): void {
  const text = input.value.trim();
  if (channel && text) {
    input.value = "";
    channel.send(text);
  }
}

Channel.fetchChannel(parseInt(useRoute().params.id as string)).then(
  (c) => {
    c = Channel.ensure(c);
    c.poll(
      (m) => {
        history.value.unshift(m);
      }
    );
    channel = c;

    fetchHistory();
  }
);
</script>

<template>
  <ChannelListBar>
    <div class="main h-100 w-100">
      <div class="channel-history d-flex flex-column-reverse overflow-y-scroll px-1 w-100" ref="channel-history"
        @scroll="scrollHistory">
        <MessageTile v-for="m in history" :author="m.author.username" :content="m.content" :key="m.id" />
      </div>
      <div class="chat-input px-3 py-4 w-100">
        <form @submit.prevent="send">
          <input type="text" class="d-block position-relative rounded-1 start-50 top-50 translate-middle w-100"
            placeholder="Message" v-model="input" @submit.prevent="send" />
        </form>
      </div>
    </div>
  </ChannelListBar>
</template>

<style lang="css" scoped>
.main {
  --chat-input-height: 50px;
}

.channel-history {
  height: calc(100% - var(--chat-input-height));
}

.chat-input {
  height: var(--chat-input-height);
}
</style>
