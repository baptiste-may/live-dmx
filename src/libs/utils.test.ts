import { describe, expect, it } from "vitest";

import { sliceString } from "./utils";

describe("sliceString", () => {
  it("returns the string unchanged when it is shorter than the maximum length", () => {
    expect(sliceString("short", 10)).toBe("short");
  });

  it("returns the string unchanged when it matches the maximum length", () => {
    expect(sliceString("exact", 5)).toBe("exact");
  });

  it("adds an ellipsis and keeps the last characters when the string is too long", () => {
    expect(sliceString("a long string", 6)).toBe("...string");
  });

  it("prefixes the entire string with an ellipsis when the maximum length is zero", () => {
    expect(sliceString("text", 0)).toBe("...text");
  });

  it("returns an empty string unchanged", () => {
    expect(sliceString("", 0)).toBe("");
    expect(sliceString("", 10)).toBe("");
  });
});
