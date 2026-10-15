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

  it("renders scale ticks with unit labels when showScale is set", () => {
    render(Fader, {
      props: {
        modelValue: 50,
        showScale: true,
        scaleStep: 25,
        scaleUnit: "%",
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    expect(screen.getByText("0%")).toBeInTheDocument();
    expect(screen.getByText("25%")).toBeInTheDocument();
    expect(screen.getByText("50%")).toBeInTheDocument();
    expect(screen.getByText("75%")).toBeInTheDocument();
    expect(screen.getByText("100%")).toBeInTheDocument();

    const strip = screen.getByText("100%").parentElement?.parentElement;
    expect(Array.from(strip?.children ?? []).map((node) => node.textContent)).toEqual([
      "100%",
      "75%",
      "50%",
      "25%",
      "0%",
    ]);

    const row = screen.getByText("100%").parentElement;
    expect(row?.children[0].className).toContain("h-px");
    expect(row?.children[0].className).toContain("w-4");
    expect(row?.children[1].textContent).toBe("100%");
  });

  it("insets the scale strip by the thumb radius on both ends", () => {
    render(Fader, {
      props: {
        modelValue: 50,
        showScale: true,
        scaleUnit: "%",
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    const strip = screen.getByText("100%").parentElement?.parentElement as HTMLElement;
    expect(strip.style.paddingTop).toBe("24px");
    expect(strip.style.paddingBottom).toBe("24px");
  });

  it("omits scale ticks without showScale", () => {
    render(Fader, {
      props: {
        modelValue: 50,
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    expect(screen.queryByText("50%")).not.toBeInTheDocument();
  });

  it("omits scale ticks when scaleStep is not positive", () => {
    render(Fader, {
      props: {
        modelValue: 50,
        showScale: true,
        scaleStep: 0,
        scaleUnit: "%",
      },
      attrs: {
        min: 0,
        max: 100,
      },
    });

    expect(screen.queryByText("50%")).not.toBeInTheDocument();
  });
});
