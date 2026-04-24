<template>
  <div>
    <p>用户：{{ user }}</p>
  </div>
</template>

<script setup lang="ts">
const user = useSupabaseUser();
</script>
