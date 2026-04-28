<template>
  <div class="flex h-screen w-screen items-center justify-center">
    <UPageCard class="w-full max-w-md" spotlight spotlight-color="primary">
      <UChatShimmer
        text="正在登入……"
        :duration="1.5"
        :spread="10"
        class="text-center text-xl"
      />
    </UPageCard>
  </div>
</template>

<script setup lang="ts">
const user = useSupabaseUser();

watch(
  user,
  () => {
    if (user.value) {
      return navigateTo({
        path: "/",
        query: {},
        hash: "",
      });
    }
  },
  { immediate: true },
);
</script>
