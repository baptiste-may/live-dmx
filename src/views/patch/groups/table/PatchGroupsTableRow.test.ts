import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { mockTableRowFixture } from "@/views/patch/__mocks__/fixtures";

import PatchGroupsTableRow from "./PatchGroupsTableRow.vue";

describe("PatchGroupsTableRow.vue", () => {
  it("should render fixture columns and emit selection update", async () => {
    const user = userEvent.setup();

    const { emitted } = render(PatchGroupsTableRow, {
      props: {
        patch: mockTableRowFixture,
        isSelected: false,
      },
    });

    expect(screen.getByText("Front Par 5")).toBeInTheDocument();
    expect(screen.getByText("LED Wash")).toBeInTheDocument();
    expect(screen.getByText("2.012")).toBeInTheDocument();

    const checkbox = screen.getByRole("checkbox");
    await user.click(checkbox);

    expect(emitted()["update-selection"]).toBeTruthy();
  });
});
