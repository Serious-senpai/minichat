<script setup lang="ts">
import { Modal } from "bootstrap";
import { ref } from "vue";
import { RouterLink } from "vue-router";

import ChannelTile from "./ChannelTile.vue";
import { Channel } from "../api/channels";
import client from "../api/client";

enum AuthState {
  LoggedIn,
  LoggedOut,
  Loading,
}

const authState = ref(AuthState.Loading);
const sideBarOpened = ref(false);
const username = ref("");
const password = ref("");

async function login(): Promise<void> {
  localStorage.setItem("username", username.value);
  localStorage.setItem("password", password.value);

  const modalElement = document.getElementById("login-modal");
  if (modalElement) {
    const loginModal = Modal.getInstance(modalElement);
    loginModal?.hide();
  }

  authState.value = AuthState.Loading;
  authState.value = await client.login() ? AuthState.LoggedIn : AuthState.LoggedOut;
}

function logout(): void {
  client.logout();
  authState.value = AuthState.LoggedOut;
}

if (Channel.channels.length === 0) {
  Channel.query();
}

if (client.user) {
  authState.value = AuthState.LoggedIn;
} else {
  client.login().then(
    (success) => {
      authState.value = success ? AuthState.LoggedIn : AuthState.LoggedOut;
    }
  );
}
</script>

<template>
  <div class="viewport d-flex vh-100" :class="{ 'show-side-bar': sideBarOpened }">
    <div class="modal" id="login-modal" tabindex="-1">
      <div class="modal-dialog modal-dialog-centered">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Login</h5>
            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
          </div>
          <div class="modal-body">
            <form @submit.prevent="login">
              <div class="mb-3">
                <label for="auth-username" class="form-label">Username</label>
                <input type="text" class="form-control" id="auth-username" v-model="username">
              </div>
              <div class="mb-3">
                <label for="auth-password" class="form-label">Password</label>
                <input type="password" class="form-control" id="auth-password" v-model="password">
              </div>
              <button type="submit" class="btn btn-primary" @click.prevent="login">Submit</button>
            </form>
          </div>
          <div class="modal-footer">
            <a class="text-decoration-none" href="#">Don't have an account? Create a new one!</a>
          </div>
        </div>
      </div>
    </div>
    <div class="sidebar bg-dark p-2 h-100 z-3">
      <div class="overflow-y-auto p-1 w-100">
        <RouterLink to="/">
          <img src="@/assets/logo.svg" alt="logo" class="d-block mx-auto mb-3" style="width: 100px;">
        </RouterLink>
        <ChannelTile v-for="c in Channel.channels" :channel_id=c.id :channel_name="c.name" :key="c.id"></ChannelTile>
      </div>
      <div class="p-1 w-100">
        <div v-if="authState === AuthState.LoggedIn">
          <button type="button" class="btn h-100 text-start w-100" data-bs-toggle="dropdown" aria-expanded="false">
            <span class="fs6-text text-info">Logged in as</span>
            <br>
            <span class="text-white">{{ client.user?.username }}</span>
          </button>
          <ul class="dropdown-menu">
            <li><a class="dropdown-item" @click.prevent="Channel.query" href="#">Refresh channel list</a></li>
            <li>
              <hr class="dropdown-divider">
            </li>
            <li><a class="dropdown-item" @click.prevent="logout" href="#">Logout</a></li>
          </ul>
        </div>
        <div v-else-if="authState === AuthState.LoggedOut">
          <button type="button" class="btn h-100 text-start w-100" data-bs-toggle="modal" data-bs-target="#login-modal">
            <span class="text-white">Login</span>
          </button>
        </div>
        <div v-else>
          <div class="spinner-border text-info" role="status">
            <span class="visually-hidden">Loading...</span>
          </div>
        </div>
      </div>
    </div>
    <div class="bg-dark bg-gradient d-flex flex-column h-100 z-0" style="flex: 1;">
      <div class="bg-black p-1 w-100" style="height: 60px;">
        <button type="button" class="btn d-block d-md-none position-relative start-0 top-50 translate-middle-y"
          @click="sideBarOpened = !sideBarOpened">
          <span class="d-block material-icons-outlined text-white">menu</span>
        </button>
      </div>
      <div class="flex-fill" @click="sideBarOpened = false">
        <slot>
          <div class="position-relative start-50 w-100 top-50 translate-middle">
            <img src="@/assets/mikucry.png" alt="mikucry"
              class="mw-100 opacity-50 p-1 position-relative start-50 translate-middle-x"
              style="mix-blend-mode: luminosity;">
            <span class="d-block position-relative text-center text-white">Where am I?</span>
          </div>
        </slot>
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.viewport {
  --sidebar-width: 300px;
  position: relative;
  left: calc(-1 * var(--sidebar-width));
  width: calc(100vw + var(--sidebar-width));
  transition: left 0.3s ease-in-out, width 0.3s ease-in-out;
}

.viewport.show-side-bar {
  left: 0;
  width: 100vw;
}

.sidebar {
  display: grid;
  grid-template-rows: 1fr auto;
  width: var(--sidebar-width);
}

@media (min-width: 768px) {
  .viewport {
    left: 0;
    width: 100vw;
  }
}
</style>
