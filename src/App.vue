<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { enable, disable } from '@tauri-apps/plugin-autostart';

interface Settings {
  source_lang: string;
  target_lang: string;
  auto_start: boolean;
  hotkey: string;
  setup_completed: boolean;
  translation_type: string;
  api_provider: string;
  api_key: string;
}

const LANGUAGES = [
  { code: 'en', name: 'English' },
  { code: 'tr', name: 'Turkish' },
  { code: 'de', name: 'German' },
  { code: 'fr', name: 'French' },
  { code: 'es', name: 'Spanish' },
  { code: 'it', name: 'Italian' },
  { code: 'pt', name: 'Portuguese' },
  { code: 'ru', name: 'Russian' },
  { code: 'ar', name: 'Arabic' },
  { code: 'ja', name: 'Japanese' },
  { code: 'ko', name: 'Korean' },
  { code: 'zh', name: 'Chinese' },
];

const API_PROVIDERS = [
  { code: 'mymemory', name: 'MyMemory (Free)', needKey: false },
  { code: 'deepL', name: 'DeepL (API Key required)', needKey: true },
  { code: 'openrouter', name: 'OpenRouter (API Key required)', needKey: true },
];

const TRANSLATION_TYPES = [
  { code: 'casual', name: 'Casual' },
  { code: 'formal', name: 'Formal' },
];

const settings = ref<Settings | null>(null);
const saved = ref(false);
const recordingHotkey = ref(false);
const hotkeyError = ref('');
const showApiKey = ref(false);
const loading = ref(true);

const selectedProvider = computed(() => {
  return API_PROVIDERS.find(p => p.code === settings.value?.api_provider);
});

onMounted(async () => {
  try {
    const data = await invoke<Settings>('get_settings');
    settings.value = {
      ...data,
      translation_type: data.translation_type || 'casual',
      api_provider: data.api_provider || 'mymemory',
      api_key: data.api_key || '',
    };
  } catch (e) {
    console.error('Failed to init app:', e);
    settings.value = {
      source_lang: 'en',
      target_lang: 'tr',
      auto_start: false,
      hotkey: 'Ctrl+Shift+T',
      setup_completed: true,
      translation_type: 'casual',
      api_provider: 'mymemory',
      api_key: '',
    };
  }
  loading.value = false;
});

const handleHotkeyRecord = (e: KeyboardEvent) => {
  if (!recordingHotkey.value) return;
  e.preventDefault();
  hotkeyError.value = '';

  const hasModifier = e.ctrlKey || e.shiftKey || e.altKey;

  if (e.key === 'Control' || e.key === 'Shift' || e.key === 'Alt' || e.key === 'Meta') {
    return;
  }

  if (!hasModifier) {
    hotkeyError.value = 'Ctrl, Shift or Alt required';
    return;
  }

  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');

  let key = e.key.toUpperCase();
  if (key === ' ') key = 'Space';
  if (key === 'CONTROL') key = 'Ctrl';
  if (key === 'SHIFT') key = 'Shift';
  if (key === 'ALT') key = 'Alt';
  if (key === 'META') key = 'Win';

  parts.push(key);
  if (settings.value) settings.value.hotkey = parts.join('+');
  recordingHotkey.value = false;
};

const handleSave = async () => {
  if (!settings.value) return;

  if (settings.value.source_lang === settings.value.target_lang) return;
  if (hotkeyError.value) return;

  const provider = API_PROVIDERS.find(p => p.code === settings.value?.api_provider);
  if (provider?.needKey && !settings.value.api_key.trim()) return;

  try {
    await invoke('update_settings', { newSettings: settings.value });

    try {
      await invoke('register_hotkey', { hotkey: settings.value.hotkey });
    } catch (e) {
      console.error('Hotkey registration failed:', e);
    }

    if (settings.value.auto_start) {
      await enable();
    } else {
      await disable();
    }

    saved.value = true;
    setTimeout(() => saved.value = false, 1500);
  } catch (e) {
    console.error('Failed to save settings:', e);
  }
};
</script>

<template>
  <div class="h-screen bg-black text-white flex flex-col">
    <!-- Header -->
    <header class="h-12 border-b border-zinc-900 flex items-center gap-2 px-4 flex-shrink-0">
      <img src="/icons/32x32.png" alt="Logo" class="w-6 h-6 rounded" />
      <span class="font-medium text-sm">QuickTranslate</span>
    </header>

    <!-- Content -->
    <main class="flex-1 p-4">
      <div class="flex flex-col gap-4">
        <!-- Source & Target Language -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">Source</label>
            <select
              v-model="settings!.source_lang"
              class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-zinc-600 transition-colors"
            >
              <option value="auto">Auto</option>
              <option v-for="lang in LANGUAGES" :key="lang.code" :value="lang.code">{{ lang.name }}</option>
            </select>
          </div>

          <div>
            <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">Target</label>
            <select
              v-model="settings!.target_lang"
              class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-zinc-600 transition-colors"
            >
              <option v-for="lang in LANGUAGES" :key="lang.code" :value="lang.code">{{ lang.name }}</option>
            </select>
          </div>
        </div>

        <!-- Translation Type -->
        <div>
          <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">Style</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="type in TRANSLATION_TYPES"
              :key="type.code"
              @click="settings!.translation_type = type.code"
              :class="[
                'p-2 rounded-lg border text-center text-sm transition-colors',
                settings!.translation_type === type.code
                  ? 'border-blue-500 bg-blue-500/10 text-white'
                  : 'border-zinc-800 bg-zinc-950 text-zinc-400 hover:border-zinc-600'
              ]"
            >
              {{ type.name }}
            </button>
          </div>
        </div>

        <!-- API Provider -->
        <div>
          <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">Service</label>
          <select
            v-model="settings!.api_provider"
            class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-zinc-600 transition-colors"
          >
            <option v-for="provider in API_PROVIDERS" :key="provider.code" :value="provider.code">
              {{ provider.name }}
            </option>
          </select>
        </div>

        <!-- API Key -->
        <div v-if="selectedProvider?.needKey">
          <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">
            <span class="flex items-center gap-1.5">API Key</span>
          </label>
          <div class="relative">
            <input
              :type="showApiKey ? 'text' : 'password'"
              v-model="settings!.api_key"
              :placeholder="settings!.api_provider === 'deepL' ? 'DeepL API Key' : 'OpenRouter API Key'"
              class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 pr-16 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-zinc-600 transition-colors"
            />
            <button
              type="button"
              @click="showApiKey = !showApiKey"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-zinc-500 hover:text-zinc-300"
            >
              {{ showApiKey ? 'Hide' : 'Show' }}
            </button>
          </div>
        </div>

        <!-- Hotkey -->
        <div>
          <label class="block text-xs text-zinc-500 uppercase tracking-wider mb-1.5">Hotkey</label>
          <button
            @click="recordingHotkey = true; hotkeyError = ''"
            @keydown="handleHotkeyRecord"
            @blur="recordingHotkey = false"
            :class="[
              'w-full bg-zinc-950 border rounded-lg px-3 py-2 text-sm transition-colors',
              recordingHotkey
                ? 'border-blue-500 text-blue-400'
                : 'border-zinc-800 text-white hover:border-zinc-600'
            ]"
          >
            {{ recordingHotkey ? 'Press a key...' : settings?.hotkey }}
          </button>
          <div v-if="hotkeyError" class="flex items-center gap-1.5 mt-1 text-amber-500 text-xs">
            {{ hotkeyError }}
          </div>
        </div>

        <!-- Auto Start -->
        <div class="flex items-center justify-between p-2.5 bg-zinc-950 rounded-lg border border-zinc-900">
          <span class="text-sm text-white">Auto start</span>
          <button
            @click="settings!.auto_start = !settings!.auto_start"
            :class="[
              'relative w-9 h-5 rounded-full transition-colors',
              settings!.auto_start ? 'bg-white' : 'bg-zinc-700'
            ]"
          >
            <div
              :class="[
                'absolute top-0.5 w-4 h-4 bg-black rounded-full transition-transform',
                settings!.auto_start ? 'translate-x-4' : 'translate-x-0.5'
              ]"
            />
          </button>
        </div>

        <!-- Save Button -->
        <button
          @click="handleSave"
          :disabled="!!hotkeyError || (selectedProvider?.needKey && !settings?.api_key.trim())"
          :class="[
            'w-full py-2.5 rounded-lg font-medium text-sm transition-colors',
            saved
              ? 'bg-green-600 text-white'
              : 'bg-white text-black hover:bg-zinc-200 disabled:opacity-50 disabled:cursor-not-allowed'
          ]"
        >
          {{ saved ? '✓ Saved' : 'Save' }}
        </button>
      </div>
    </main>

    <!-- Footer -->
    <footer class="p-3 text-center border-t border-zinc-900 flex-shrink-0">
      <p class="text-xs text-zinc-600">{{ settings?.hotkey }} to translate</p>
    </footer>
  </div>

  <!-- Loading -->
  <div v-if="loading" class="h-screen bg-black flex items-center justify-center">
    <div class="w-5 h-5 border-2 border-zinc-800 border-t-zinc-400 rounded-full animate-spin" />
  </div>
</template>
