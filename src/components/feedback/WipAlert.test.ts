import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import WipAlert from "./WipAlert.vue";

describe("WipAlert.vue", () => {
  it("should render alert message when mounted", () => {
    render(WipAlert);

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("This app is still in development.")).toBeInTheDocument();
  });
});
