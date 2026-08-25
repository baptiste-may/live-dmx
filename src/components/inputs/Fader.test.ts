import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Fader from "./Fader.vue";

describe("Fader.vue", () => {
  it("renders range input with initial modelValue", () => {
    render(Fader, {
      props: {
        modelValue: 50,
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    const range = screen.getByRole("slider");
    expect(range).toBeInTheDocument();
    expect(range).toHaveValue("50");
  });

  it("emits update:modelValue when slider value changes", async () => {
    const { emitted } = render(Fader, {
      props: {
        modelValue: 20,
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    const range = screen.getByRole("slider");
    await fireEvent.update(range, "75");

    expect(emitted("update:modelValue")).toBeDefined();
    expect(emitted("update:modelValue")).toContainEqual([75]);
  });

  it("binds extra attributes to the inner Range component", () => {
    render(Fader, {
      props: {
        modelValue: 0,
      },
      attrs: {
        min: 10,
        max: 200,
        step: 5,
      },
    });

    const range = screen.getByRole("slider");
    expect(range).toHaveAttribute("min", "10");
    expect(range).toHaveAttribute("max", "200");
    expect(range).toHaveAttribute("step", "5");
  });
});
