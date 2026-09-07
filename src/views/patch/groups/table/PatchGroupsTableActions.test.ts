import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";

import PatchGroupsTableActions from "./PatchGroupsTableActions.vue";

describe("PatchGroupsTableActions.vue", () => {
  it("should disable buttons when no fixtures are selected", () => {
    render(PatchGroupsTableActions, {
      props: {
        selectedFixtures: [],
      },
    });

    const buttons = screen.getAllByRole("button");
    expect(buttons[0]).toBeDisabled();
    expect(buttons[1]).toBeDisabled();
  });

  it("should open edit modal when configure button is clicked", async () => {
    const user = userEvent.setup();
    render(PatchGroupsTableActions, {
      props: {
        selectedFixtures: [1, 2],
      },
    });

    const patchGroups = usePatchGroups();
    const buttons = screen.getAllByRole("button");
    expect(buttons[0]).not.toBeDisabled();

    await user.click(buttons[0]);
    expect(patchGroups.openModal).toHaveBeenCalledWith([1, 2]);
  });
});
