import { describe, expect, it } from "vitest";

import { formatDate, parseDate } from "./date";

describe("date utility", () => {
  describe("parseDate", () => {
    it("should return Date instance as-is if valid", () => {
      const date = new Date(1740501255000);
      expect(parseDate(date)).toBe(date);
    });

    it("should return null for invalid Date instance", () => {
      const date = new Date("invalid date");
      expect(parseDate(date)).toBeNull();
    });

    it("should parse number timestamps in milliseconds", () => {
      const result = parseDate(1740501255000);
      expect(result).toBeInstanceOf(Date);
      expect(result?.getTime()).toBe(1740501255000);
    });

    it("should parse numeric string timestamps in milliseconds", () => {
      const result = parseDate("1740501255000");
      expect(result).toBeInstanceOf(Date);
      expect(result?.getTime()).toBe(1740501255000);
    });

    it("should parse ISO 8601 date strings", () => {
      const isoStr = "2026-02-25T16:34:15.000Z";
      const result = parseDate(isoStr);
      expect(result).toBeInstanceOf(Date);
      expect(result?.toISOString()).toBe(isoStr);
    });

    it("should return null for empty string or whitespace", () => {
      expect(parseDate("")).toBeNull();
      expect(parseDate("   ")).toBeNull();
    });

    it("should return null for null or undefined", () => {
      expect(parseDate(null)).toBeNull();
      expect(parseDate(undefined)).toBeNull();
    });

    it("should return null for invalid date strings", () => {
      expect(parseDate("not-a-valid-date")).toBeNull();
    });
  });

  describe("formatDate", () => {
    it("should format numeric string timestamp without throwing or returning 'Invalid Date'", () => {
      const formatted = formatDate(
        "1740501255000",
        { dateStyle: "short", timeStyle: "short" },
        "en-US",
      );
      expect(formatted).not.toContain("Invalid");
      expect(formatted.length).toBeGreaterThan(0);
    });

    it("should format number timestamp", () => {
      const formatted = formatDate(
        1740501255000,
        { dateStyle: "short", timeStyle: "short" },
        "en-US",
      );
      expect(formatted).not.toContain("Invalid");
      expect(formatted.length).toBeGreaterThan(0);
    });

    it("should format ISO string", () => {
      const formatted = formatDate(
        "2026-02-25T16:34:15.000Z",
        { dateStyle: "short", timeStyle: "short" },
        "en-US",
      );
      expect(formatted).not.toContain("Invalid");
      expect(formatted.length).toBeGreaterThan(0);
    });

    it("should return empty string for null, undefined, or invalid inputs", () => {
      expect(formatDate(null)).toBe("");
      expect(formatDate(undefined)).toBe("");
      expect(formatDate("")).toBe("");
      expect(formatDate("invalid-date-string")).toBe("");
    });
  });
});
