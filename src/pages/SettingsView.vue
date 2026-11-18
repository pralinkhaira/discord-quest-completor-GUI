<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, SessionState } from '@/types/types';
import { useGlobalState } from '@/composables/app-state';

const { addLog } = useGlobalState();
const settings = ref<AppSettings>({ minimize_to_tray: true });
const activeSession = ref<SessionState | null>(null);
const isLoading = ref(true);

async function loadSettings() {
  try {
    const res = await invoke<AppSettings>('get_tray_settings');
    settings.value = res;
  } catch (err) {
    console.error('Failed to load tray settings:', err);
  }
}

async function loadActiveSession() {
  try {
    const session = await invoke<SessionState | null>('get_active_session');
    activeSession.value = session;
  } catch (err) {
    console.error('Failed to load active session:', err);
  }
}

async function onToggleMinimizeToTray() {
  try {
    const updated = await invoke<AppSettings>('set_minimize_to_tray', {
      enabled: settings.value.minimize_to_tray,
    });
    settings.value = updated;
    addLog('info', `Minimize to tray preference saved: ${updated.minimize_to_tray}`);
  } catch (err) {
    console.error('Failed to update tray settings:', err);
    addLog('error', 'Failed to update minimize to tray preference');
  }
}

async function hideToTray() {
  try {
    addLog('info', 'Hiding window to system tray');
    await invoke('hide_window');
  } catch (err) {
    console.error('Failed to hide window:', err);
  }
}

function formatTimestamp(ts: number): string {
  if (!ts) return 'N/A';
  return new Date(ts * 1000).toLocaleString();
}

function formatElapsed(startTs: number): string {
  if (!startTs) return '0s';
  const now = Math.floor(Date.now() / 1000);
  const diff = Math.max(0, now - startTs);
  const hours = Math.floor(diff / 3600);
  const minutes = Math.floor((diff % 3600) / 60);
  const seconds = diff % 60;
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

onMounted(async () => {
  isLoading.value = true;
  await Promise.all([loadSettings(), loadActiveSession()]);
  isLoading.value = false;
});
</script>

<template>
  <div class="container mx-auto px-4 py-6 max-w-4xl">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Application Settings</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
        Configure system tray behavior, background execution, and session persistence.
      </p>
    </div>

    <!-- System Tray Settings Card -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">System Tray Integration</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Keep Discord Quest Completer running quietly in the background without cluttering the taskbar.
          </p>
        </div>
        <div class="flex items-center">
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              v-model="settings.minimize_to_tray"
              @change="onToggleMinimizeToTray"
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-indigo-600"></div>
          </label>
        </div>
      </div>

      <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700 text-sm text-gray-600 dark:text-gray-300 space-y-2">
        <div class="flex items-start space-x-2">
          <svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
          <span><strong>Minimize to Tray on Close:</strong> Clicking the window close button hides the window to the system tray instead of terminating the app.</span>
        </div>
        <div class="flex items-start space-x-2">
          <svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
          <span><strong>Tray Context Menu:</strong> Right-click the tray icon to choose <em>Open Application</em>, <em>Hide Application</em>, or <em>Exit Application</em>.</span>
        </div>
        <div class="flex items-start space-x-2">
          <svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
          <span><strong>Quick Restore:</strong> Double-click the tray icon at any time to restore and focus the main window.</span>
        </div>
      </div>

      <div class="mt-4 pt-3 flex justify-end">
        <button
          @click="hideToTray"
          class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 text-sm font-medium rounded-md transition-colors"
        >
          Hide to Tray Now
        </button>
      </div>
    </div>

    <!-- Active Session & Power Resilience Card -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Session Persistence & Power Handling</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
        Discord Quest Completer uses absolute timestamp tracking to ensure quest progress continues across sleep and hibernation.
      </p>

      <div v-if="activeSession" class="bg-indigo-50 dark:bg-indigo-900/30 border border-indigo-200 dark:border-indigo-800 rounded-md p-4 space-y-2 text-sm">
        <div class="flex items-center justify-between">
          <span class="font-medium text-indigo-900 dark:text-indigo-200">Active Game:</span>
          <span class="font-bold text-indigo-700 dark:text-indigo-300">{{ activeSession.game_name }} (App ID: {{ activeSession.app_id }})</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="font-medium text-indigo-900 dark:text-indigo-200">Session Started:</span>
          <span class="text-indigo-700 dark:text-indigo-300">{{ formatTimestamp(activeSession.start_timestamp) }}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="font-medium text-indigo-900 dark:text-indigo-200">Elapsed Quest Time:</span>
          <span class="font-mono font-bold text-indigo-700 dark:text-indigo-300">{{ formatElapsed(activeSession.start_timestamp) }}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="font-medium text-indigo-900 dark:text-indigo-200">State:</span>
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            Active & Persisted to Disk
          </span>
        </div>
      </div>
      <div v-else class="bg-gray-50 dark:bg-gray-700/50 rounded-md p-4 text-sm text-gray-500 dark:text-gray-400">
        No active game quest session in progress. Start any game from the Home tab to begin tracking.
      </div>

      <div class="mt-4 text-xs text-gray-500 dark:text-gray-400 space-y-1">
        <p>• <strong>Hibernation Resilience:</strong> Time spent asleep or hibernated is computed as <code>elapsed = now - session_start</code>. Hibernation time counts fully towards quest completion.</p>
        <p>• <strong>Crash & Restart Recovery:</strong> If the application is closed or restarted, the active session is recovered from <code>active_session.json</code> and Discord Rich Presence is reconnected with the original start timestamp.</p>
      </div>
    </div>

    <!-- About & License Card -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">About & Attribution</h2>
      <p class="text-sm text-gray-600 dark:text-gray-300">
        Discord Quest Completer is an open-source utility designed to emulate game presence for Discord quests.
      </p>
      <div class="mt-3 text-xs text-gray-500 dark:text-gray-400 space-y-1">
        <p>Original Author: Terence Mark (MIT License)</p>
        <p>Engine: Tauri v2 + Rust + Vue 3</p>
        <p>Attribution and license preserved in accordance with open-source guidelines.</p>
      </div>
    </div>
  </div>
</template>
