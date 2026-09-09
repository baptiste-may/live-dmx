import { render } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import FixtureIcon from "./FixtureIcon.vue";

describe("FixtureIcon.vue", () => {
  it("should render correct SVG class for PAR category", () => {
    const { container } = render(FixtureIcon, {
      props: { category: "PAR" },
    });
    expect(container.querySelector(".lucide-fixture-par")).not.toBeNull();
  });

  it("should render correct SVG class for Moving Head category", () => {
    const { container } = render(FixtureIcon, {
      props: { category: "Moving Head" },
    });
    expect(container.querySelector(".lucide-fixture-moving-head")).not.toBeNull();
  });

  it("should render correct SVG class for Scanner category", () => {
    const { container } = render(FixtureIcon, {
      props: { category: "Scanner" },
    });
    expect(container.querySelector(".lucide-fixture-scanner")).not.toBeNull();
  });

  it("should render correct SVG class for Smoke / Hazer category", () => {
    const { container } = render(FixtureIcon, {
      props: { category: ["Hazer", "Smoke"] },
    });
    expect(container.querySelector(".lucide-fixture-smoke")).not.toBeNull();
  });

  it("should fallback to generic SVG class for unknown category", () => {
    const { container } = render(FixtureIcon, {
      props: { category: "Unknown Type" },
    });
    expect(container.querySelector(".lucide-fixture-generic")).not.toBeNull();
  });

  it("should render new categories (Water, Wind, Confetti, Emitter, Geyser, Pyro)", () => {
    const cases: [string, string][] = [
      ["Water", "lucide-fixture-water"],
      ["Wind", "lucide-fixture-wind"],
      ["Confetti", "lucide-fixture-confetti"],
      ["Emitter", "lucide-fixture-emitter"],
      ["Geyser", "lucide-fixture-geyser"],
      ["Pyro", "lucide-fixture-pyro"],
    ];

    for (const [category, expectedClass] of cases) {
      const { container } = render(FixtureIcon, {
        props: { category },
      });
      expect(container.querySelector(`.${expectedClass}`), category).not.toBeNull();
    }
  });

  it("should apply custom size and strokeWidth attributes", () => {
    const { container } = render(FixtureIcon, {
      props: { category: "PAR", size: 32, strokeWidth: 3, color: "#ff0000" },
    });
    const svg = container.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(svg?.getAttribute("width")).toBe("32");
    expect(svg?.getAttribute("height")).toBe("32");
    expect(svg?.getAttribute("stroke-width")).toBe("3");
    expect(svg?.getAttribute("stroke")).toBe("#ff0000");
  });
});
