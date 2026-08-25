import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useLiveButtons } from "./live-buttons";

describe("live-buttons store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should update flash state and invoke tauri command", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.isFlashActive(1, "master")).toBe(false);

    await liveButtons.setFlash(1, "master", true);

    expect(liveButtons.isFlashActive(1, "master")).toBe(true);
    expect(invoke).toHaveBeenCalledWith("live_set_flash", {
      groupId: 1,
      flashType: "master",
      active: true,
    });
  });

  it("should update button state and toggle button", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.isButtonActive("btn-1")).toBe(false);

    await liveButtons.setButtonState("btn-1", true);

    expect(liveButtons.isButtonActive("btn-1")).toBe(true);
    expect(invoke).toHaveBeenCalledWith("live_set_button", {
      buttonId: "btn-1",
      active: true,
    });

    await liveButtons.toggleButton("btn-1");

    expect(liveButtons.isButtonActive("btn-1")).toBe(false);
    expect(invoke).toHaveBeenCalledWith("live_set_button", {
      buttonId: "btn-1",
      active: false,
    });
  });

  it("should manage group effect parameters and invoke tauri command", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.getGroupEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
    });

    await liveButtons.setGroupEffectType(1, "chase");

    expect(liveButtons.getGroupEffect(1).effect).toBe("chase");
    expect(invoke).toHaveBeenCalledWith("live_set_group_effect", {
      groupId: 1,
      effect: "chase",
      alignment: null,
      bounce: false,
    });

    await liveButtons.setGroupEffectAlignment(1, "left");

    expect(liveButtons.getGroupEffect(1).alignment).toBe("left");
    expect(invoke).toHaveBeenCalledWith("live_set_group_effect", {
      groupId: 1,
      effect: "chase",
      alignment: "left",
      bounce: false,
    });

    await liveButtons.setGroupEffectBounce(1, true);

    expect(liveButtons.getGroupEffect(1).bounce).toBe(true);
    expect(invoke).toHaveBeenCalledWith("live_set_group_effect", {
      groupId: 1,
      effect: "chase",
      alignment: "left",
      bounce: true,
    });
  });

  it("should manage group master and strobe rates", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.getGroupMaster(1)).toBe(100);

    await liveButtons.setGroupMaster(1, 80);
    expect(liveButtons.getGroupMaster(1)).toBe(80);
    expect(invoke).toHaveBeenCalledWith("live_set_group_master", {
      groupId: 1,
      level: 80,
    });

    await liveButtons.setStrobeRate(75);
    expect(liveButtons.strobeRate).toBe(75);
    expect(invoke).toHaveBeenCalledWith("live_set_strobe_rate", { rate: 75 });

    await liveButtons.setRandomStrobeRate(90);
    expect(liveButtons.randomStrobeRate).toBe(90);
    expect(invoke).toHaveBeenCalledWith("live_set_random_strobe_rate", { rate: 90 });

    expect(liveButtons.effectBpm).toBe(60);
    await liveButtons.setEffectBpm(140);
    expect(liveButtons.effectBpm).toBe(140);
    expect(invoke).toHaveBeenCalledWith("live_set_effect_bpm", { bpm: 140 });
  });

  it("should manage group colors and invoke tauri command", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.getGroupColor(1)).toBe("white");

    await liveButtons.setGroupColor(1, "red");
    expect(liveButtons.getGroupColor(1)).toBe("red");
    expect(invoke).toHaveBeenCalledWith("live_set_group_color", {
      groupId: 1,
      color: "red",
    });

    await liveButtons.setAllGroupsColor("blue", [1, 2, 3]);
    expect(liveButtons.getGroupColor(1)).toBe("blue");
    expect(liveButtons.getGroupColor(2)).toBe("blue");
    expect(liveButtons.getGroupColor(3)).toBe("blue");
    expect(liveButtons.getGroupColor(0)).toBe("blue");
    expect(invoke).toHaveBeenCalledWith("live_set_all_groups_color", {
      color: "blue",
    });

    await liveButtons.resetGroupColor(1);
    expect(liveButtons.getGroupColor(1)).toBe("blue"); // Falls back to group 0 (which was set to blue)
    expect(invoke).toHaveBeenCalledWith("live_set_group_color", {
      groupId: 1,
      color: null,
    });
  });

  it("should manage color fade speed and order", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.colorFadeSpeed).toBe(1);
    expect(liveButtons.colorFadeOrder).toBeNull();

    await liveButtons.setColorFadeSpeed(2.5);
    expect(liveButtons.colorFadeSpeed).toBe(2.5);
    expect(invoke).toHaveBeenCalledWith("live_set_color_fade_speed", { speed: 2.5 });

    await liveButtons.setColorFadeOrder("left");
    expect(liveButtons.colorFadeOrder).toBe("left");
    expect(invoke).toHaveBeenCalledWith("live_set_color_fade_order", { order: "left" });

    await liveButtons.setColorFadeOrder("wing");
    expect(liveButtons.colorFadeOrder).toBe("wing");
    expect(invoke).toHaveBeenCalledWith("live_set_color_fade_order", { order: "wing" });

    await liveButtons.setColorFadeOrder(null);
    expect(liveButtons.colorFadeOrder).toBeNull();
    expect(invoke).toHaveBeenCalledWith("live_set_color_fade_order", { order: null });
  });

  it("should manage group secondary colors and invoke tauri command", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.getGroupSecondaryColor(1)).toBeNull();

    await liveButtons.setGroupSecondaryColor(1, "yellow");
    expect(liveButtons.getGroupSecondaryColor(1)).toBe("yellow");
    expect(invoke).toHaveBeenCalledWith("live_set_group_secondary_color", {
      groupId: 1,
      color: "yellow",
    });

    await liveButtons.resetGroupSecondaryColor(1);
    expect(liveButtons.getGroupSecondaryColor(1)).toBeNull();
    expect(invoke).toHaveBeenCalledWith("live_set_group_secondary_color", {
      groupId: 1,
      color: null,
    });
  });

  it("should manage group color effect parameters and invoke tauri command", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.getGroupColorEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
    });

    await liveButtons.setGroupColorEffectType(1, "pulse");
    expect(liveButtons.getGroupColorEffect(1).effect).toBe("pulse");
    expect(invoke).toHaveBeenCalledWith("live_set_group_color_effect", {
      groupId: 1,
      effect: "pulse",
      alignment: null,
      bounce: false,
    });

    await liveButtons.setGroupColorEffectAlignment(1, "wing");
    expect(liveButtons.getGroupColorEffect(1).alignment).toBe("wing");
    expect(invoke).toHaveBeenCalledWith("live_set_group_color_effect", {
      groupId: 1,
      effect: "pulse",
      alignment: "wing",
      bounce: false,
    });

    await liveButtons.setGroupColorEffectBounce(1, true);
    expect(liveButtons.getGroupColorEffect(1).bounce).toBe(true);
    expect(invoke).toHaveBeenCalledWith("live_set_group_color_effect", {
      groupId: 1,
      effect: "pulse",
      alignment: "wing",
      bounce: true,
    });

    await liveButtons.resetGroupColorEffect(1);
    expect(liveButtons.getGroupColorEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
    });
    expect(invoke).toHaveBeenCalledWith("live_reset_group_color_effect", { groupId: 1 });
  });

  it("should manage color effect bpm", async () => {
    const liveButtons = useLiveButtons();

    expect(liveButtons.colorEffectBpm).toBe(60);
    await liveButtons.setColorEffectBpm(120);
    expect(liveButtons.colorEffectBpm).toBe(120);
    expect(invoke).toHaveBeenCalledWith("live_set_color_effect_bpm", { bpm: 120 });
  });

  it("should reset group effect", async () => {
    const liveButtons = useLiveButtons();

    await liveButtons.setGroupEffectType(1, "pulse");
    await liveButtons.resetGroupEffect(1);

    expect(liveButtons.getGroupEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
    });
    expect(invoke).toHaveBeenCalledWith("live_reset_group_effect", { groupId: 1 });
  });

  it("should reset all states", async () => {
    const liveButtons = useLiveButtons();

    await liveButtons.setFlash(1, "strobe", true);
    await liveButtons.setButtonState("b1", true);
    await liveButtons.setGroupColor(1, "cyan");
    await liveButtons.setGroupSecondaryColor(1, "magenta");
    await liveButtons.setGroupColorEffectType(1, "chase");
    await liveButtons.setColorEffectBpm(140);
    await liveButtons.setColorFadeSpeed(2.5);
    await liveButtons.setColorFadeOrder("right");
    await liveButtons.resetAll();

    expect(liveButtons.isFlashActive(1, "strobe")).toBe(false);
    expect(liveButtons.isButtonActive("b1")).toBe(false);
    expect(liveButtons.getGroupColor(1)).toBe("white");
    expect(liveButtons.getGroupSecondaryColor(1)).toBeNull();
    expect(liveButtons.getGroupColorEffect(1).effect).toBeNull();
    expect(liveButtons.colorEffectBpm).toBe(60);
    expect(liveButtons.colorFadeSpeed).toBe(1);
    expect(liveButtons.colorFadeOrder).toBeNull();
    expect(invoke).toHaveBeenCalledWith("live_reset_all_buttons");
  });
});
