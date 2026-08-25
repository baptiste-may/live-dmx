import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "@/stores/app";
import { useRecentShows } from "@/stores/recent-shows";

import RecentShowsList from "./RecentShowsList.vue";

describe("RecentShowsList.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render recent header and refresh button", () => {
    render(RecentShowsList);

    expect(screen.getByText("Recent")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Refresh shows" })).toBeInTheDocument();
  });

  it("should trigger refresh when refresh button is clicked", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    render(RecentShowsList);

    const refreshButton = screen.getByRole("button", { name: "Refresh shows" });
    await user.click(refreshButton);

    expect(recentShowsStore.load).toHaveBeenCalled();
  });

  it("should render recent shows list and load on click", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    const appStore = useApp();

    recentShowsStore.shows = [
      {
        displayName: "Concert Tour 2026",
        path: "/shows/concert.ldmx",
        updatedAt: new Date(),
      },
    ];

    render(RecentShowsList);

    expect(screen.getByText("Concert Tour 2026")).toBeInTheDocument();
    expect(screen.getByText("/shows/concert.ldmx")).toBeInTheDocument();

    await user.click(screen.getByText("Concert Tour 2026"));

    expect(appStore.loadShow).toHaveBeenCalledWith("/shows/concert.ldmx");
  });

  it("should render formatted date without 'Invalid date' when updatedAt is a timestamp string", () => {
    const recentShowsStore = useRecentShows();

    recentShowsStore.shows = [
      {
        displayName: "Festival 2026",
        path: "/shows/festival.ldmx",
        updatedAt: "1740501255000",
      },
    ];

    render(RecentShowsList);

    expect(screen.getByText("Festival 2026")).toBeInTheDocument();
    expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
  });
});
