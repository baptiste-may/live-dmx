import { describe, expect, it } from "vitest";

import FixtureIconComponent from "./FixtureIcon.vue";
import { FixtureIcon } from "./index";

describe("icons/index", () => {
  it("should export FixtureIcon", () => {
    expect(FixtureIcon).toBe(FixtureIconComponent);
  });
});
