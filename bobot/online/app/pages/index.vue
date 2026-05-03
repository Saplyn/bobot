<template>
  <div>
    <div>{{ user }}</div>
  </div>
</template>

<script setup lang="ts">
const supabase = useSupabaseClient();

const {
  data: { user },
} = await supabase.auth.getUser();
</script>
