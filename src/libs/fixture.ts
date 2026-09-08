import type { NamedChannel } from "@/types/fixtures";
import type { PatchedFixture } from "@/types/patch";

export const failedPreviewKeys = new Set<string>();

/**
 * Type guard to check if a DMX channel reference is a NamedChannel object.
 */
export function isNamedChannel(ch: unknown): ch is NamedChannel {
  return (
    typeof ch === "object" &&
    ch !== null &&
    "name" in ch &&
    typeof (ch as NamedChannel).name === "string"
  );
}

export function formatChannelName(ch: unknown): string {
  if (typeof ch === "string" && ch.trim() !== "") {
    return ch;
  }
  if (isNamedChannel(ch)) {
    return ch.name;
  }
  if (ch === null) {
    return "No Function / Reserved";
  }
  return "";
}

/**
 * Checks whether a channel reference represents a color channel.
 */
export function isColorChannel(ch: unknown): boolean {
  if (ch === null || ch === undefined) return false;

  let name = "";
  if (typeof ch === "string") {
    name = ch;
  } else if (typeof ch === "object") {
    const obj = ch as Record<string, unknown>;
    const cap = obj.capability as Record<string, unknown> | undefined;
    if (cap) {
      if (
        cap.type === "ColorIntensity" ||
        cap.type === "ColorPreset" ||
        cap.color !== undefined ||
        cap.colors !== undefined
      ) {
        return true;
      }
    }
    const caps = obj.capabilities as Record<string, unknown>[] | undefined;
    if (Array.isArray(caps)) {
      if (
        caps.some(
          (c) =>
            c.type === "ColorIntensity" ||
            c.type === "ColorPreset" ||
            c.color !== undefined ||
            c.colors !== undefined,
        )
      ) {
        return true;
      }
    }

    if (typeof obj.name === "string") {
      name = obj.name;
    } else if (typeof obj.channel === "string") {
      name = obj.channel;
    }
  }

  const trimmed = name.trim();
  if (!trimmed) return false;
  const lower = trimmed.toLowerCase();

  // Exclude fine / LSB / 16-bit
  if (
    lower.includes("fine") ||
    lower.includes("lsb") ||
    lower.includes("16 bit") ||
    lower.includes("16bit") ||
    lower.includes("16-bit")
  ) {
    return false;
  }

  // Exact single letter or single letter + number: "r", "g", "b", "w", "a", "c", "m", "y", "r1", etc.
  if (/^[rgbwacmy](\s*\d+)?$/i.test(lower)) {
    return true;
  }

  // UV / ultraviolet (using word boundary for 'uv' to avoid false positives like 'Dimmer Curve')
  if (/\buv\b/i.test(lower) || lower.includes("ultraviolet")) {
    return true;
  }

  // CTO / CTB / CTC color temperature filters (using word boundary to avoid words like 'sector')
  if (/\b(cto|ctb|ctc)\b/i.test(lower)) {
    return true;
  }

  // Lime (using word boundary)
  if (/\blime\b/i.test(lower)) {
    return true;
  }

  // Core color keywords
  return (
    lower.includes("red") ||
    lower.includes("green") ||
    lower.includes("blue") ||
    lower.includes("white") ||
    lower.includes("amber") ||
    lower.includes("cyan") ||
    lower.includes("magenta") ||
    lower.includes("yellow") ||
    lower.includes("color") ||
    lower.includes("colour")
  );
}

/**
 * Checks whether a patched fixture has color capability based on its mode channels.
 */
export function fixtureHasColorCapability(fixture: PatchedFixture): boolean {
  if (!fixture || !fixture.mode || !Array.isArray(fixture.mode.channels)) {
    return false;
  }
  return fixture.mode.channels.some((ch) => isColorChannel(ch));
}

/**
 * Checks whether a fixture group has color capability based on its assigned fixtures.
 */
export function groupHasColorCapability(groupId: number, fixtures: PatchedFixture[]): boolean {
  if (groupId === 0) return false;
  return fixtures.some(
    (fixture) => fixture.groups.includes(groupId) && fixtureHasColorCapability(fixture),
  );
}
