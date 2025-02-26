<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { onBeforeRouteUpdate, useRoute } from "vue-router";

import { Channel, Message } from "../api/channels";
import ChannelListBar from "../components/ChannelListBar.vue";
import MessageTile from "../components/MessageTile.vue";

class PageState {
  public readonly input = ref("");
  public readonly history = ref([] as Message[]);
  public readonly historyContainer = useTemplateRef("channel-history");

  public channel: Channel | null = null;
  public loading = false;
  public oldestMessageId = 1n << 63n;
  public noMoreMessages = false;
  public websocket: WebSocket | null = null;

  public constructor(id?: string) {
    Channel.fetchChannel(BigInt(id ?? useRoute().params.id as string)).then(
      (c) => {
        c = Channel.ensure(c);
        this.channel = c;
        this.websocket = c.poll((m) => this.history.value.unshift(m));

        fetchHistory();
      },
    );
  }
}

async function fetchHistory(): Promise<void> {
  const channel = state.channel;
  if (channel) {
    const older = await channel.history({
      newest: true,
      beforeId: state.oldestMessageId - 1n,
      afterId: 0n,
      limit: 50,
    });

    if (older.length === 0) {
      state.noMoreMessages = true;
      return;
    }

    state.history.value.push(...older);
    state.oldestMessageId = older[older.length - 1].id;
  }
}

async function scrollHistory(): Promise<void> {
  const element = state.historyContainer.value;
  if (element) {
    const limit = element.clientHeight - element.scrollHeight; // negative value, since we're scrolling up
    if (element.scrollTop - limit < 10 && !state.loading && !state.noMoreMessages) {
      state.loading = true;
      await fetchHistory();
      state.loading = false;
    }
  }
}

function send(): void {
  const text = state.input.value.trim();
  if (state.channel && text) {
    state.input.value = "";
    state.channel.send(text);
  }

  const element = state.historyContainer.value;
  if (element) {
    element.scrollTop = element.scrollHeight;
  }
}

let state = new PageState();

onBeforeRouteUpdate(
  (to) => {
    if (state.websocket) {
      state.websocket.close();
    }

    state = new PageState(to.params.id);
  },
);
</script>

<template>
  <ChannelListBar v-slot="slotProps" :key="useRoute().params.id as string">
    <div class="main h-100 w-100">
      <div class="channel-history d-flex flex-column-reverse overflow-y-scroll px-1 w-100" ref="channel-history" @scroll="scrollHistory">
        <MessageTile v-for="m in state.history.value" :message="Message.ensure(m)" :key="m.id.toString()" />
      </div>
      <div class="chat-input px-3 py-4 w-100">
        <form @submit.prevent="send">
          <input type="text" class="d-block position-relative rounded-1 start-50 top-50 translate-middle w-100" :disabled="!slotProps.loggedIn" :placeholder="slotProps.loggedIn ? `Message` : `Please login first`" v-model="state.input.value" @submit.prevent="send" />
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
