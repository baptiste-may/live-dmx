import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import NumberIcon from "./NumberIcon.vue";

describe("NumberIcon.vue", () => {
  it("should render given number in a badge", () => {
    render(NumberIcon, {
      props: {
        number: 2,
      },
    });
    expect(screen.getByText("2")).toBeInTheDocument();
  });
});
