import { describe, expect, it } from "vitest";

import { formatChannelName, isNamedChannel } from "./fixture";

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
});
