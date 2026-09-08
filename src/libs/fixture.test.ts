import { describe, expect, it } from "vitest";

import type { PatchedFixture } from "@/types/patch";

import {
  fixtureHasColorCapability,
  formatChannelName,
  groupHasColorCapability,
  isColorChannel,
  isNamedChannel,
} from "./fixture";

describe("fixture helper", () => {
  describe("isNamedChannel", () => {
    it("should return true for valid NamedChannel object", () => {
      expect(isNamedChannel({ name: "Dimmer" })).toBe(true);
      expect(isNamedChannel({ name: "Red", type: "SingleColor" })).toBe(true);
    });

    it("should return false for primitive types, null, or invalid objects", () => {
      expect(isNamedChannel("Dimmer")).toBe(false);
      expect(isNamedChannel(null)).toBe(false);
      expect(isNamedChannel(undefined)).toBe(false);
      expect(isNamedChannel({})).toBe(false);
      expect(isNamedChannel({ name: 123 })).toBe(false);
    });
  });

  describe("formatChannelName", () => {
    it("should format string channel names", () => {
      expect(formatChannelName("Shutter")).toBe("Shutter");
    });

    it("should format NamedChannel objects", () => {
      expect(formatChannelName({ name: "Strobe" })).toBe("Strobe");
    });

    it("should return 'No Function / Reserved' for null", () => {
      expect(formatChannelName(null)).toBe("No Function / Reserved");
    });

    it("should return empty string for empty string or unrecognized format", () => {
      expect(formatChannelName("  ")).toBe("");
      expect(formatChannelName({})).toBe("");
      expect(formatChannelName(123)).toBe("");
    });
  });

  describe("isColorChannel", () => {
    it("should recognize common color channel names", () => {
      expect(isColorChannel("Red")).toBe(true);
      expect(isColorChannel("Green")).toBe(true);
      expect(isColorChannel("Blue")).toBe(true);
      expect(isColorChannel("White")).toBe(true);
      expect(isColorChannel("Warm White")).toBe(true);
      expect(isColorChannel("Amber")).toBe(true);
      expect(isColorChannel("Cyan")).toBe(true);
      expect(isColorChannel("Magenta")).toBe(true);
      expect(isColorChannel("Yellow")).toBe(true);
      expect(isColorChannel("UV")).toBe(true);
      expect(isColorChannel("Ultraviolet")).toBe(true);
      expect(isColorChannel("Lime")).toBe(true);
      expect(isColorChannel("Color Wheel")).toBe(true);
      expect(isColorChannel("Colour Wheel")).toBe(true);
      expect(isColorChannel("Color Macro")).toBe(true);
      expect(isColorChannel("Linear CTO")).toBe(true);
      expect(isColorChannel("CTB")).toBe(true);
      expect(isColorChannel("CTC")).toBe(true);
    });

    it("should recognize single-letter color channels", () => {
      expect(isColorChannel("R")).toBe(true);
      expect(isColorChannel("G")).toBe(true);
      expect(isColorChannel("B")).toBe(true);
      expect(isColorChannel("W")).toBe(true);
      expect(isColorChannel("A")).toBe(true);
      expect(isColorChannel("C")).toBe(true);
      expect(isColorChannel("M")).toBe(true);
      expect(isColorChannel("Y")).toBe(true);
      expect(isColorChannel("R1")).toBe(true);
      expect(isColorChannel("G 2")).toBe(true);
    });

    it("should recognize NamedChannel and capability objects", () => {
      expect(isColorChannel({ name: "Red" })).toBe(true);
      expect(isColorChannel({ channel: "Green" })).toBe(true);
      expect(
        isColorChannel({
          name: "Custom",
          capability: { type: "ColorIntensity" },
        }),
      ).toBe(true);
      expect(
        isColorChannel({
          name: "Preset",
          capabilities: [{ type: "ColorPreset" }],
        }),
      ).toBe(true);
      expect(
        isColorChannel({
          name: "Wheel",
          capability: { color: "#ff0000" },
        }),
      ).toBe(true);
    });

    it("should reject non-color channels including false positive traps", () => {
      expect(isColorChannel("Dimmer")).toBe(false);
      expect(isColorChannel("Dimmer Curve")).toBe(false);
      expect(isColorChannel("Sector")).toBe(false);
      expect(isColorChannel("Pan")).toBe(false);
      expect(isColorChannel("Tilt")).toBe(false);
      expect(isColorChannel("Strobe")).toBe(false);
      expect(isColorChannel("Shutter")).toBe(false);
      expect(isColorChannel("Gobo")).toBe(false);
      expect(isColorChannel("Prism")).toBe(false);
      expect(isColorChannel("Zoom")).toBe(false);
      expect(isColorChannel("Focus")).toBe(false);
      expect(isColorChannel("Red Fine")).toBe(false);
      expect(isColorChannel("Green LSB")).toBe(false);
      expect(isColorChannel("Blue (16 bit)")).toBe(false);
      expect(isColorChannel("")).toBe(false);
      expect(isColorChannel("   ")).toBe(false);
      expect(isColorChannel(null)).toBe(false);
      expect(isColorChannel(undefined)).toBe(false);
    });
  });

  describe("fixtureHasColorCapability", () => {
    it("should return true when fixture has at least one color channel", () => {
      const fixture: PatchedFixture = {
        id: 1,
        groups: [1],
        name: "LED Par",
        universe: 1,
        address: 1,
        fixtureType: { name: "LED Par" },
        mode: {
          name: "4ch",
          channels: ["Dimmer", "Red", "Green", "Blue"],
        },
      };

      expect(fixtureHasColorCapability(fixture)).toBe(true);
    });

    it("should return false when fixture has no color channels", () => {
      const fixture: PatchedFixture = {
        id: 2,
        groups: [1],
        name: "Traditional Dimmer",
        universe: 1,
        address: 10,
        fixtureType: { name: "Dimmer" },
        mode: {
          name: "1ch",
          channels: ["Dimmer"],
        },
      };

      expect(fixtureHasColorCapability(fixture)).toBe(false);
    });

    it("should return false for invalid or missing mode", () => {
      const fixture = {
        id: 3,
        groups: [1],
        name: "Invalid",
        universe: 1,
        address: 20,
        fixtureType: { name: "Invalid" },
        mode: { name: "empty", channels: [] },
      } as PatchedFixture;

      expect(fixtureHasColorCapability(fixture)).toBe(false);
    });
  });

  describe("groupHasColorCapability", () => {
    const fixtures: PatchedFixture[] = [
      {
        id: 1,
        groups: [1],
        name: "Color Fixture",
        universe: 1,
        address: 1,
        fixtureType: { name: "Par" },
        mode: { name: "RGB", channels: ["Red", "Green", "Blue"] },
      },
      {
        id: 2,
        groups: [2],
        name: "Dimmer Fixture",
        universe: 1,
        address: 10,
        fixtureType: { name: "Dimmer" },
        mode: { name: "1ch", channels: ["Dimmer"] },
      },
    ];

    it("should return true for a group with color fixtures", () => {
      expect(groupHasColorCapability(1, fixtures)).toBe(true);
    });

    it("should return false for a group with only non-color fixtures", () => {
      expect(groupHasColorCapability(2, fixtures)).toBe(false);
    });

    it("should return false for a group with no fixtures", () => {
      expect(groupHasColorCapability(3, fixtures)).toBe(false);
    });

    it("should return false for group 0 ('All')", () => {
      expect(groupHasColorCapability(0, fixtures)).toBe(false);
    });
  });
});
