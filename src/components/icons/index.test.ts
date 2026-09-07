import { describe, expect, it } from "vitest";

import FixtureIconComponent from "./FixtureIcon.vue";
import { FixtureIcon, FourIcon, NumberIcon, ThreeIcon, TwoIcon } from "./index";
import NumberIconComponent from "./NumberIcon.vue";

describe("icons/index", () => {
  it("should export FixtureIcon and NumberIcon", () => {
    expect(FixtureIcon).toBe(FixtureIconComponent);
    expect(NumberIcon).toBe(NumberIconComponent);
  });

  it("should provide functional number icon factories", () => {
    expect(typeof TwoIcon).toBe("function");
    expect(typeof ThreeIcon).toBe("function");
    expect(typeof FourIcon).toBe("function");
  });
});
