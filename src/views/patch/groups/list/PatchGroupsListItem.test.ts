import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";

import PatchGroupsListItem from "./PatchGroupsListItem.vue";

describe("PatchGroupsListItem.vue", () => {
  it("should render group name and allow selecting it", async () => {
    const user = userEvent.setup();
    render(PatchGroupsListItem, {
      props: {
        group: { id: 1, name: "Stage Left" },
      },
    });

    const groupsStore = usePatchGroups();
    const groupButton = screen.getByRole("button", { name: "Stage Left" });
    expect(groupButton).toBeInTheDocument();

    await user.click(groupButton);
    expect(groupsStore.select).toHaveBeenCalledWith(1);
  });
});
