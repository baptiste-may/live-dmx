import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import { ToastType, useToastStore } from "@/stores/toast.ts";
import type {
  AlignmentType,
  ColorEffectType,
  ColorFadeOrder,
  FlashType,
  GroupColorEffectState,
  GroupEffectState,
  IntensityEffectType,
  LiveButtonsState,
} from "@/types/live";

export const getDefaultGroupEffectState = (): GroupEffectState => ({
  effect: null,
  alignment: null,
  bounce: false,
});

export const defaultGroupEffectState: GroupEffectState = getDefaultGroupEffectState();

export const getDefaultGroupColorEffectState = (): GroupColorEffectState => ({
  effect: null,
  alignment: null,
  bounce: false,
});

export const defaultGroupColorEffectState: GroupColorEffectState =
  getDefaultGroupColorEffectState();

export const getDefaultLiveButtonsState = (): LiveButtonsState => ({
  flashStates: {},
  buttonStates: {},
  groupEffects: {},
  groupColorEffects: {},
  groupMasters: {},
  groupColors: { 0: "white" },
  groupSecondaryColors: {},
  strobeRate: 50,
  randomStrobeRate: 50,
  effectBpm: 60,
  colorEffectBpm: 60,
  colorFadeSpeed: 1,
  colorFadeOrder: null,
});

export const defaultLiveButtonsState: LiveButtonsState = getDefaultLiveButtonsState();

function handleActionError(error: unknown) {
  useToastStore().addToast(`Live control failed: ${String(error)}`, ToastType.ERROR);
}

export const useLiveButtons = defineStore("live-buttons", {
  state: (): LiveButtonsState => getDefaultLiveButtonsState(),
  getters: {
    isFlashActive:
      (state) =>
      (groupId: number, flashType: FlashType): boolean => {
        return state.flashStates[groupId]?.[flashType] ?? false;
      },
    isButtonActive:
      (state) =>
      (buttonId: string): boolean => {
        return state.buttonStates[buttonId] ?? false;
      },
    getGroupEffect:
      (state) =>
      (groupId: number): GroupEffectState => {
        return state.groupEffects[groupId] ?? { ...defaultGroupEffectState };
      },
    getGroupColorEffect:
      (state) =>
      (groupId: number): GroupColorEffectState => {
        return state.groupColorEffects[groupId] ?? { ...defaultGroupColorEffectState };
      },
    getGroupMaster:
      (state) =>
      (groupId: number): number => {
        return state.groupMasters[groupId] ?? 100;
      },
    getGroupColor:
      (state) =>
      (groupId: number): string => {
        return state.groupColors[groupId] ?? state.groupColors[0] ?? "white";
      },
    getGroupSecondaryColor:
      (state) =>
      (groupId: number): string | null => {
        return state.groupSecondaryColors[groupId] ?? state.groupSecondaryColors[0] ?? null;
      },
  },
  actions: {
    async setFlash(groupId: number, flashType: FlashType, active: boolean) {
      if (!this.flashStates[groupId]) {
        this.flashStates[groupId] = {
          master: false,
          strobe: false,
          randomStrobe: false,
        };
      }
      const prev = this.flashStates[groupId][flashType];
      this.flashStates[groupId][flashType] = active;
      try {
        await invoke("live_set_flash", { groupId, flashType, active });
      } catch (error) {
        this.flashStates[groupId][flashType] = prev;
        handleActionError(error);
      }
    },
    async setButtonState(buttonId: string, active: boolean) {
      const prev = this.buttonStates[buttonId];
      this.buttonStates[buttonId] = active;
      try {
        await invoke("live_set_button", { buttonId, active });
      } catch (error) {
        if (prev !== undefined) {
          this.buttonStates[buttonId] = prev;
        } else {
          delete this.buttonStates[buttonId];
        }
        handleActionError(error);
      }
    },
    async toggleButton(buttonId: string) {
      const currentState = this.isButtonActive(buttonId);
      await this.setButtonState(buttonId, !currentState);
    },
    ensureGroupEffect(groupId: number): GroupEffectState {
      if (!this.groupEffects[groupId]) {
        this.groupEffects[groupId] = { ...defaultGroupEffectState };
      }
      return this.groupEffects[groupId];
    },
    async setGroupEffectType(groupId: number, effect: IntensityEffectType | null) {
      const current = this.ensureGroupEffect(groupId);
      const prev = current.effect;
      current.effect = effect;
      try {
        await invoke("live_set_group_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.effect = prev;
        handleActionError(error);
      }
    },
    async setGroupEffectAlignment(groupId: number, alignment: AlignmentType | null) {
      const current = this.ensureGroupEffect(groupId);
      const prev = current.alignment;
      current.alignment = alignment;
      try {
        await invoke("live_set_group_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.alignment = prev;
        handleActionError(error);
      }
    },
    async setGroupEffectBounce(groupId: number, bounce: boolean) {
      const current = this.ensureGroupEffect(groupId);
      const prev = current.bounce;
      current.bounce = bounce;
      try {
        await invoke("live_set_group_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.bounce = prev;
        handleActionError(error);
      }
    },
    async setGroupMaster(groupId: number, level: number) {
      const prev = this.groupMasters[groupId];
      this.groupMasters[groupId] = level;
      try {
        await invoke("live_set_group_master", { groupId, level });
      } catch (error) {
        if (prev !== undefined) {
          this.groupMasters[groupId] = prev;
        } else {
          delete this.groupMasters[groupId];
        }
        handleActionError(error);
      }
    },
    async setGroupColor(groupId: number, color: string | null) {
      const prev = this.groupColors[groupId];
      this.groupColors[groupId] = color;
      try {
        await invoke("live_set_group_color", { groupId, color });
      } catch (error) {
        if (prev !== undefined) {
          this.groupColors[groupId] = prev;
        } else {
          delete this.groupColors[groupId];
        }
        handleActionError(error);
      }
    },
    async setGroupSecondaryColor(groupId: number, color: string | null) {
      const prev = this.groupSecondaryColors[groupId];
      this.groupSecondaryColors[groupId] = color;
      try {
        await invoke("live_set_group_secondary_color", { groupId, color });
      } catch (error) {
        if (prev !== undefined) {
          this.groupSecondaryColors[groupId] = prev;
        } else {
          delete this.groupSecondaryColors[groupId];
        }
        handleActionError(error);
      }
    },
    async setAllGroupsColor(color: string | null, groupIds?: number[]) {
      const prevColors = { ...this.groupColors };
      if (groupIds && groupIds.length > 0) {
        for (const id of groupIds) {
          this.groupColors[id] = color;
        }
      }
      this.groupColors[0] = color;
      try {
        await invoke("live_set_all_groups_color", { color });
      } catch (error) {
        this.groupColors = prevColors;
        handleActionError(error);
      }
    },
    async resetGroupColor(groupId: number) {
      await this.setGroupColor(groupId, null);
    },
    async resetGroupSecondaryColor(groupId: number) {
      await this.setGroupSecondaryColor(groupId, null);
    },
    ensureGroupColorEffect(groupId: number): GroupColorEffectState {
      if (!this.groupColorEffects[groupId]) {
        this.groupColorEffects[groupId] = { ...defaultGroupColorEffectState };
      }
      return this.groupColorEffects[groupId];
    },
    async setGroupColorEffectType(groupId: number, effect: ColorEffectType | null) {
      const current = this.ensureGroupColorEffect(groupId);
      const prev = current.effect;
      current.effect = effect;
      try {
        await invoke("live_set_group_color_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.effect = prev;
        handleActionError(error);
      }
    },
    async setGroupColorEffectAlignment(groupId: number, alignment: AlignmentType | null) {
      const current = this.ensureGroupColorEffect(groupId);
      const prev = current.alignment;
      current.alignment = alignment;
      try {
        await invoke("live_set_group_color_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.alignment = prev;
        handleActionError(error);
      }
    },
    async setGroupColorEffectBounce(groupId: number, bounce: boolean) {
      const current = this.ensureGroupColorEffect(groupId);
      const prev = current.bounce;
      current.bounce = bounce;
      try {
        await invoke("live_set_group_color_effect", {
          groupId,
          effect: current.effect,
          alignment: current.alignment,
          bounce: current.bounce,
        });
      } catch (error) {
        current.bounce = prev;
        handleActionError(error);
      }
    },
    async resetGroupColorEffect(groupId: number) {
      const prev = this.groupColorEffects[groupId]
        ? { ...this.groupColorEffects[groupId] }
        : undefined;
      this.groupColorEffects[groupId] = { ...defaultGroupColorEffectState };
      try {
        await invoke("live_reset_group_color_effect", { groupId });
      } catch (error) {
        if (prev) {
          this.groupColorEffects[groupId] = prev;
        } else {
          delete this.groupColorEffects[groupId];
        }
        handleActionError(error);
      }
    },
    async setStrobeRate(rate: number) {
      const prev = this.strobeRate;
      this.strobeRate = rate;
      try {
        await invoke("live_set_strobe_rate", { rate });
      } catch (error) {
        this.strobeRate = prev;
        handleActionError(error);
      }
    },
    async setRandomStrobeRate(rate: number) {
      const prev = this.randomStrobeRate;
      this.randomStrobeRate = rate;
      try {
        await invoke("live_set_random_strobe_rate", { rate });
      } catch (error) {
        this.randomStrobeRate = prev;
        handleActionError(error);
      }
    },
    async setEffectBpm(bpm: number) {
      const prev = this.effectBpm;
      this.effectBpm = bpm;
      try {
        await invoke("live_set_effect_bpm", { bpm });
      } catch (error) {
        this.effectBpm = prev;
        handleActionError(error);
      }
    },
    async setColorEffectBpm(bpm: number) {
      const prev = this.colorEffectBpm;
      this.colorEffectBpm = bpm;
      try {
        await invoke("live_set_color_effect_bpm", { bpm });
      } catch (error) {
        this.colorEffectBpm = prev;
        handleActionError(error);
      }
    },
    async setColorFadeSpeed(speed: number) {
      const prev = this.colorFadeSpeed;
      this.colorFadeSpeed = speed;
      try {
        await invoke("live_set_color_fade_speed", { speed });
      } catch (error) {
        this.colorFadeSpeed = prev;
        handleActionError(error);
      }
    },
    async setColorFadeOrder(order: ColorFadeOrder | null) {
      const prev = this.colorFadeOrder;
      this.colorFadeOrder = order;
      try {
        await invoke("live_set_color_fade_order", { order });
      } catch (error) {
        this.colorFadeOrder = prev;
        handleActionError(error);
      }
    },
    async resetGroupEffect(groupId: number) {
      const prev = this.groupEffects[groupId] ? { ...this.groupEffects[groupId] } : undefined;
      this.groupEffects[groupId] = { ...defaultGroupEffectState };
      try {
        await invoke("live_reset_group_effect", { groupId });
      } catch (error) {
        if (prev) {
          this.groupEffects[groupId] = prev;
        } else {
          delete this.groupEffects[groupId];
        }
        handleActionError(error);
      }
    },
    async resetAll() {
      const prev = {
        flashStates: { ...this.flashStates },
        buttonStates: { ...this.buttonStates },
        groupEffects: { ...this.groupEffects },
        groupColorEffects: { ...this.groupColorEffects },
        groupMasters: { ...this.groupMasters },
        groupColors: { ...this.groupColors },
        groupSecondaryColors: { ...this.groupSecondaryColors },
        strobeRate: this.strobeRate,
        randomStrobeRate: this.randomStrobeRate,
        effectBpm: this.effectBpm,
        colorEffectBpm: this.colorEffectBpm,
        colorFadeSpeed: this.colorFadeSpeed,
        colorFadeOrder: this.colorFadeOrder,
      };
      this.$reset();
      try {
        await invoke("live_reset_all_buttons");
      } catch (error) {
        this.flashStates = prev.flashStates;
        this.buttonStates = prev.buttonStates;
        this.groupEffects = prev.groupEffects;
        this.groupColorEffects = prev.groupColorEffects;
        this.groupMasters = prev.groupMasters;
        this.groupColors = prev.groupColors;
        this.groupSecondaryColors = prev.groupSecondaryColors;
        this.strobeRate = prev.strobeRate;
        this.randomStrobeRate = prev.randomStrobeRate;
        this.effectBpm = prev.effectBpm;
        this.colorEffectBpm = prev.colorEffectBpm;
        this.colorFadeSpeed = prev.colorFadeSpeed;
        this.colorFadeOrder = prev.colorFadeOrder;
        handleActionError(error);
      }
    },
  },
});
