<script setup lang="ts">
import { BUTTONS } from "../ts/buttons";

const emit = defineEmits<{ wrap: [open: string, close: string] }>();
</script>

<template>
  <div class="toolbar">
    <button
      v-for="b in BUTTONS"
      :key="b.label"
      class="toolbar__button"
      @click="emit('wrap', b.open, b.close)"
    >
      {{ b.label }}
    </button>
  </div>
</template>
