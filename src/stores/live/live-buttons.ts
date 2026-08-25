import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

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
      this.flashStates[groupId][flashType] = active;
      await invoke("live_set_flash", { groupId, flashType, active }).catch(() => {});
    },
    async setButtonState(buttonId: string, active: boolean) {
      this.buttonStates[buttonId] = active;
      await invoke("live_set_button", { buttonId, active }).catch(() => {});
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
      current.effect = effect;
      await invoke("live_set_group_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async setGroupEffectAlignment(groupId: number, alignment: AlignmentType | null) {
      const current = this.ensureGroupEffect(groupId);
      current.alignment = alignment;
      await invoke("live_set_group_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async setGroupEffectBounce(groupId: number, bounce: boolean) {
      const current = this.ensureGroupEffect(groupId);
      current.bounce = bounce;
      await invoke("live_set_group_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async setGroupMaster(groupId: number, level: number) {
      this.groupMasters[groupId] = level;
      await invoke("live_set_group_master", { groupId, level }).catch(() => {});
    },
    async setGroupColor(groupId: number, color: string | null) {
      this.groupColors[groupId] = color;
      await invoke("live_set_group_color", { groupId, color }).catch(() => {});
    },
    async setGroupSecondaryColor(groupId: number, color: string | null) {
      this.groupSecondaryColors[groupId] = color;
      await invoke("live_set_group_secondary_color", { groupId, color }).catch(() => {});
    },
    async setAllGroupsColor(color: string | null, groupIds?: number[]) {
      if (groupIds && groupIds.length > 0) {
        for (const id of groupIds) {
          this.groupColors[id] = color;
        }
      }
      this.groupColors[0] = color;
      await invoke("live_set_all_groups_color", { color }).catch(() => {});
    },
    async resetGroupColor(groupId: number) {
      this.groupColors[groupId] = null;
      await invoke("live_set_group_color", { groupId, color: null }).catch(() => {});
    },
    async resetGroupSecondaryColor(groupId: number) {
      this.groupSecondaryColors[groupId] = null;
      await invoke("live_set_group_secondary_color", { groupId, color: null }).catch(() => {});
    },
    ensureGroupColorEffect(groupId: number): GroupColorEffectState {
      if (!this.groupColorEffects[groupId]) {
        this.groupColorEffects[groupId] = { ...defaultGroupColorEffectState };
      }
      return this.groupColorEffects[groupId];
    },
    async setGroupColorEffectType(groupId: number, effect: ColorEffectType | null) {
      const current = this.ensureGroupColorEffect(groupId);
      current.effect = effect;
      await invoke("live_set_group_color_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async setGroupColorEffectAlignment(groupId: number, alignment: AlignmentType | null) {
      const current = this.ensureGroupColorEffect(groupId);
      current.alignment = alignment;
      await invoke("live_set_group_color_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async setGroupColorEffectBounce(groupId: number, bounce: boolean) {
      const current = this.ensureGroupColorEffect(groupId);
      current.bounce = bounce;
      await invoke("live_set_group_color_effect", {
        groupId,
        effect: current.effect,
        alignment: current.alignment,
        bounce: current.bounce,
      }).catch(() => {});
    },
    async resetGroupColorEffect(groupId: number) {
      this.groupColorEffects[groupId] = { ...defaultGroupColorEffectState };
      await invoke("live_reset_group_color_effect", { groupId }).catch(() => {});
    },
    async setStrobeRate(rate: number) {
      this.strobeRate = rate;
      await invoke("live_set_strobe_rate", { rate }).catch(() => {});
    },
    async setRandomStrobeRate(rate: number) {
      this.randomStrobeRate = rate;
      await invoke("live_set_random_strobe_rate", { rate }).catch(() => {});
    },
    async setEffectBpm(bpm: number) {
      this.effectBpm = bpm;
      await invoke("live_set_effect_bpm", { bpm }).catch(() => {});
    },
    async setColorEffectBpm(bpm: number) {
      this.colorEffectBpm = bpm;
      await invoke("live_set_color_effect_bpm", { bpm }).catch(() => {});
    },
    async setColorFadeSpeed(speed: number) {
      this.colorFadeSpeed = speed;
      await invoke("live_set_color_fade_speed", { speed }).catch(() => {});
    },
    async setColorFadeOrder(order: ColorFadeOrder | null) {
      this.colorFadeOrder = order;
      await invoke("live_set_color_fade_order", { order }).catch(() => {});
    },
    async resetGroupEffect(groupId: number) {
      this.groupEffects[groupId] = { ...defaultGroupEffectState };
      await invoke("live_reset_group_effect", { groupId }).catch(() => {});
    },
    async resetAll() {
      this.$reset();
      await invoke("live_reset_all_buttons").catch(() => {});
    },
  },
});
