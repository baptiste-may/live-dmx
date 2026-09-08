import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Knob from "./Knob.vue";

describe("Knob.vue", () => {
  it("renders with slider accessibility attributes and value", () => {
    render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 100,
        label: "Intensity",
        unit: "%",
      },
    });

    const slider = screen.getByRole("slider", { name: "Intensity" });
    expect(slider).toBeInTheDocument();
    expect(slider).toHaveAttribute("aria-valuenow", "50");
    expect(slider).toHaveAttribute("aria-valuemin", "0");
    expect(slider).toHaveAttribute("aria-valuemax", "100");
    expect(slider).toHaveAttribute("aria-valuetext", "50%");
    expect(screen.getByText("Intensity")).toBeInTheDocument();
    expect(screen.getByText("50%")).toBeInTheDocument();
  });

  it("renders with custom ariaLabel without visible label", () => {
    render(Knob, {
      props: {
        modelValue: 100,
        min: 0,
        max: 100,
        ariaLabel: "Width",
        unit: "%",
      },
    });

    const slider = screen.getByRole("slider", { name: "Width" });
    expect(slider).toBeInTheDocument();
    expect(screen.queryByText("Width")).not.toBeInTheDocument();
    expect(screen.getByText("100%")).toBeInTheDocument();
  });

  it("emits update:modelValue on keyboard navigation", async () => {
    const user = userEvent.setup();
    const { emitted } = render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 100,
        step: 5,
      },
    });

    const slider = screen.getByRole("slider");
    slider.focus();

    await user.keyboard("{ArrowUp}");
    expect(emitted("update:modelValue")).toContainEqual([55]);

    await user.keyboard("{ArrowDown}");
    expect(emitted("update:modelValue")).toContainEqual([45]);

    await user.keyboard("{Home}");
    expect(emitted("update:modelValue")).toContainEqual([0]);

    await user.keyboard("{End}");
    expect(emitted("update:modelValue")).toContainEqual([100]);
  });

  it("emits update:modelValue on mouse wheel scroll", async () => {
    const { emitted } = render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 100,
        step: 1,
      },
    });

    const slider = screen.getByRole("slider");
    await fireEvent.wheel(slider, { deltaY: -100 });

    expect(emitted("update:modelValue")).toContainEqual([51]);

    await fireEvent.wheel(slider, { deltaY: -100, shiftKey: true });
    expect(emitted("update:modelValue")).toContainEqual([50.1]);
  });

  it("emits fine-adjustment update:modelValue on Shift+keyboard navigation", async () => {
    const user = userEvent.setup();
    const { emitted } = render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 100,
        step: 1,
      },
    });

    const slider = screen.getByRole("slider");
    slider.focus();

    await user.keyboard("{Shift>}{ArrowUp}{/Shift}");
    expect(emitted("update:modelValue")).toContainEqual([50.1]);
  });

  it("emits update:modelValue on pointer drag", async () => {
    const { emitted } = render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 200,
      },
    });

    const slider = screen.getByRole("slider");
    await fireEvent.pointerDown(slider, { clientY: 200, pointerId: 1 });
    await fireEvent.pointerMove(slider, { clientY: 100, pointerId: 1 }); // dragged UP by 100px
    await fireEvent.pointerUp(slider, { pointerId: 1 });

    expect(emitted("update:modelValue")).toBeDefined();
    const lastValue = (emitted("update:modelValue") as number[][]).pop();
    expect(lastValue?.[0]).toBeGreaterThan(50);
  });

  it("resets value on double click", async () => {
    const { emitted } = render(Knob, {
      props: {
        modelValue: 75,
        min: 0,
        max: 100,
        defaultValue: 25,
      },
    });

    const slider = screen.getByRole("slider");
    await fireEvent.dblClick(slider);

    expect(emitted("update:modelValue")).toEqual([[25]]);
  });

  it("does not emit update:modelValue when disabled", async () => {
    const user = userEvent.setup();
    const { emitted } = render(Knob, {
      props: {
        modelValue: 50,
        min: 0,
        max: 100,
        disabled: true,
      },
    });

    const slider = screen.getByRole("slider");
    slider.focus();
    await user.keyboard("{ArrowUp}");

    expect(emitted("update:modelValue")).toBeUndefined();
  });
});
