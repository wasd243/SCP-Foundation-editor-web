<script setup lang="ts">
import { ref } from "vue";
import { BUTTONS } from "../ts/buttons";

const isToolbarClosed = ref(false);
const emit = defineEmits<{ wrap: [open: string, close: string] }>();

function moveToolbar() {
  isToolbarClosed.value = !isToolbarClosed.value;
}
</script>

<template>
  <div class="toolbar" :class="{ 'toolbar--closed': isToolbarClosed }">
    <template v-if="!isToolbarClosed">
      <button
        v-for="b in BUTTONS"
        :key="b.label"
        class="toolbar__button"
        @click="emit('wrap', b.open, b.close)"
      >
        {{ b.label }}
      </button>
    </template>

    <button class="toolbar__close" @click="moveToolbar">
      {{ isToolbarClosed ? "▼" : "▲" }}
    </button>
  </div>
</template>

<style lang="sass" scoped>
.toolbar--closed
  position: fixed
  top: 4px
  right: 16px
  z-index: 10
  height: auto
  padding: 0
  background: transparent
  border: none

.toolbar__close
  margin-left: auto
</style>
