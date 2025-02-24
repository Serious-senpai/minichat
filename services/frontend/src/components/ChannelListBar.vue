<script setup lang="ts">
import { Modal } from "bootstrap";
import { reactive, ref } from "vue";
import { RouterLink } from "vue-router";

import ChannelTile from "./ChannelTile.vue";
import { Channel } from "../api/channels";
import client from "../api/client";

enum AuthState {
  LoggedIn,
  LoggedOut,
  Loading,
}

const channels = ref(Channel.channels);
const authState = ref(AuthState.Loading);
const sideBarOpened = ref(false);
const authForm = reactive({
  username: "",
  password: "",
});
const createChannelForm = reactive({
  channelName: "",
  channelDescription: "",
});
const loginPopup = ref(true);

function closeLoginModal(): void {
  const modalElement = document.getElementById("login-modal");
  if (modalElement) {
    const loginModal = Modal.getInstance(modalElement);
    loginModal?.hide();
  }
}

function closeCreateChannelModal(): void {
  const modalElement = document.getElementById("create-channel-modal");
  if (modalElement) {
    const createChannelModal = Modal.getInstance(modalElement);
    createChannelModal?.hide();
  }
}

async function login(): Promise<void> {
  localStorage.setItem("username", authForm.username);
  localStorage.setItem("password", authForm.password);

  closeLoginModal();

  authState.value = AuthState.Loading;
  authState.value = await client.login() ? AuthState.LoggedIn : AuthState.LoggedOut;
}

async function register(): Promise<void> {
  closeLoginModal();
  authState.value = AuthState.Loading;
  const status = await client.register(authForm.username, authForm.password);
  authState.value = AuthState.LoggedOut;

  alert(status.message);
}

function logout(): void {
  client.logout();
  authState.value = AuthState.LoggedOut;
}

function queryChannels(): void {
  Channel.query().then(
    (c) => {
      channels.value = c;
    }
  );
}

async function createChannel(): Promise<void> {
  closeCreateChannelModal();
  await Channel.create(createChannelForm.channelName, createChannelForm.channelDescription);
  queryChannels();
}

if (channels.value.length === 0) {
  queryChannels();
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
          <div v-if="loginPopup">
            <div class="modal-header">
              <h5 class="modal-title">Login</h5>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <form @submit.prevent="login">
                <div class="mb-3">
                  <label for="login-username" class="form-label">Username</label>
                  <input type="text" class="form-control" id="login-username" v-model="authForm.username">
                </div>
                <div class="mb-3">
                  <label for="login-password" class="form-label">Password</label>
                  <input type="password" class="form-control" id="login-password" v-model="authForm.password">
                </div>
                <button type="submit" class="btn btn-primary" @click.prevent="login">Login</button>
              </form>
            </div>
            <div class="modal-footer">
              <a class="text-decoration-none" @click.prevent="loginPopup = false" href="#">Don't have an account? Create a new one!</a>
            </div>
          </div>
          <div v-else>
            <div class="modal-header">
              <h5 class="modal-title">Create a new account</h5>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <form @submit.prevent="register">
                <div class="mb-3">
                  <label for="register-username" class="form-label">Username</label>
                  <input type="text" class="form-control" id="register-username" v-model="authForm.username">
                </div>
                <div class="mb-3">
                  <label for="register-password" class="form-label">Password</label>
                  <input type="password" class="form-control" id="register-password" v-model="authForm.password">
                </div>
                <button type="submit" class="btn btn-primary" @click.prevent="register">Register</button>
              </form>
            </div>
            <div class="modal-footer">
              <a class="text-decoration-none" @click.prevent="loginPopup = true" href="#">Login using an existing account instead?</a>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="modal" id="create-channel-modal" tabindex="-1">
      <div class="modal-dialog modal-dialog-centered">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Create a new channel</h5>
            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
          </div>
          <div class="modal-body">
            <form @submit.prevent="createChannel">
              <div class="mb-3">
                <label for="channel-name" class="form-label">Channel name</label>
                <input type="text" class="form-control" id="channel-name" v-model="createChannelForm.channelName">
              </div>
              <div class="mb-3">
                <label for="channel-description" class="form-label">Description</label>
                <textarea class="form-control" id="channel-description" v-model="createChannelForm.channelDescription"></textarea>
              </div>
              <button type="submit" class="btn btn-primary" @click.prevent="createChannel">Create</button>
            </form>
          </div>
        </div>
      </div>
    </div>
    <div class="sidebar bg-dark p-2 h-100 z-3">
      <div class="overflow-y-auto p-1 w-100">
        <RouterLink to="/">
          <img src="@/assets/logo.svg" alt="logo" class="d-block mx-auto mb-3" style="width: 100px;">
        </RouterLink>
        <ChannelTile v-for="c in channels" :id=c.id :name="c.name" :key="c.id"></ChannelTile>
      </div>
      <div class="p-1 w-100">
        <div v-if="authState === AuthState.LoggedIn">
          <button type="button" class="btn h-100 text-start w-100" data-bs-toggle="dropdown" aria-expanded="false">
            <span class="fs-6 text-info">Logged in as</span>
            <br>
            <span class="fs-5 text-white">{{ client.user?.username }}</span>
          </button>
          <ul class="dropdown-menu">
            <li><a class="dropdown-item" @click.prevent="queryChannels" href="#">Refresh channel list</a></li>
            <li><a class="dropdown-item" data-bs-toggle="modal" data-bs-target="#create-channel-modal" href="#">Create a new channel</a></li>
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
    <div class="main-area bg-dark bg-gradient flex-grow-1 h-100 z-0">
      <div class="appbar bg-black p-1 w-100">
        <button type="button" class="btn d-block d-md-none position-relative start-0 top-50 translate-middle-y" @click="sideBarOpened = !sideBarOpened">
          <span class="d-block material-icons-outlined text-white">menu</span>
        </button>
      </div>
      <div class="content w-100" @click="sideBarOpened = false">
        <slot>
          <div class="position-relative start-50 w-100 top-50 translate-middle">
            <img src="@/assets/mikucry.png" alt="mikucry" class="mw-100 opacity-50 p-1 position-relative start-50 translate-middle-x" style="mix-blend-mode: luminosity;">
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

.main-area {
  --appbar-height: 60px;
}

.appbar {
  height: var(--appbar-height);
}

.content {
  height: calc(100% - var(--appbar-height));
}

@media (min-width: 768px) {
  .viewport {
    left: 0;
    width: 100vw;
  }
}
</style>
