import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import NumberStepper from "./NumberStepper.vue";

describe("NumberStepper.vue", () => {
  it("renders minus and plus buttons and current value", () => {
    render(NumberStepper, {
      props: {
        modelValue: 5,
        min: 1,
        max: 10,
      },
    });

    expect(screen.getByRole("button", { name: "Decrease" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Increase" })).toBeInTheDocument();
    expect(screen.getByRole("spinbutton")).toHaveValue(5);
  });

  it("emits updated value when decrease button is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(NumberStepper, {
      props: {
        modelValue: 5,
        min: 1,
        max: 10,
      },
    });

    await user.click(screen.getByRole("button", { name: "Decrease" }));

    expect(emitted("update:modelValue")).toEqual([[4]]);
  });

  it("emits updated value when increase button is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(NumberStepper, {
      props: {
        modelValue: 5,
        min: 1,
        max: 10,
      },
    });

    await user.click(screen.getByRole("button", { name: "Increase" }));

    expect(emitted("update:modelValue")).toEqual([[6]]);
  });

  it("disables decrease button when value is at min", () => {
    render(NumberStepper, {
      props: {
        modelValue: 1,
        min: 1,
        max: 10,
      },
    });

    expect(screen.getByRole("button", { name: "Decrease" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Increase" })).not.toBeDisabled();
  });

  it("renders NumberFlow with current modelValue", () => {
    render(NumberStepper, {
      props: {
        modelValue: 5,
        min: 1,
        max: 10,
      },
    });

    expect(screen.getByRole("spinbutton")).toHaveValue(5);
  });
});
