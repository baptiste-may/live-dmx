import { attachConsole } from "@tauri-apps/plugin-log";

export async function initLogger(): Promise<() => void> {
  try {
    return await attachConsole();
  } catch {
    return () => {
      // Ignored in non-Tauri / test environments
    };
  }
}
