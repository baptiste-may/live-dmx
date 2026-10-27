import { invoke } from "@tauri-apps/api/core";
import userEvent from "@testing-library/user-event";
import { render, screen, waitFor } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { formatDate } from "@/libs/date";
import { useApp } from "@/stores/app";
import { useRecentShows } from "@/stores/recent-shows";

import RecentShowsList from "./RecentShowsList.vue";

const timestamp = "1740501255000";

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

  it("should keep the refresh button disabled while the list loads", async () => {
    let resolveLoad: ((shows: unknown) => void) | undefined;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveLoad = resolve;
        }),
    );

    render(RecentShowsList);
    await nextTick();

    const refreshButton = screen.getByRole("button", { name: "Refresh shows" });
    expect(refreshButton).toBeDisabled();

    resolveLoad?.([]);

    await waitFor(() => expect(refreshButton).toBeEnabled());
  });

  it("should render recent shows list with details and load on click", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    const appStore = useApp();

    recentShowsStore.shows = [
      {
        displayName: "Concert Tour 2026",
        path: "/shows/concert.ldmx",
        updatedAt: timestamp,
        available: true,
      },
    ];

    render(RecentShowsList);

    expect(screen.getByText("Concert Tour 2026")).toBeInTheDocument();
    expect(screen.getByText("/shows/concert.ldmx")).toBeInTheDocument();
    expect(screen.getByText("/shows/concert.ldmx")).toHaveAttribute("title", "/shows/concert.ldmx");
    expect(screen.getByText(formatDate(timestamp))).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Remove Concert Tour 2026 from recent shows" }),
    ).toBeInTheDocument();

    await user.click(screen.getByText("Concert Tour 2026"));

    expect(appStore.loadShow).toHaveBeenCalledWith("/shows/concert.ldmx");
  });

  it("should disable opening an unavailable show but keep removal enabled", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    const appStore = useApp();

    recentShowsStore.shows = [
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: timestamp,
        available: false,
      },
    ];

    render(RecentShowsList);

    expect(screen.getByText("File not found")).toBeInTheDocument();

    const openButton = screen.getByText("Moved show").closest("button");
    expect(openButton).toBeDisabled();

    await user.click(screen.getByText("Moved show"));
    expect(appStore.loadShow).not.toHaveBeenCalled();

    const removeButton = screen.getByRole("button", {
      name: "Remove Moved show from recent shows",
    });
    expect(removeButton).toBeEnabled();

    await user.click(removeButton);
    expect(recentShowsStore.remove).toHaveBeenCalledWith("/shows/moved.ldmx");
  });

  it("should remove a show without opening it", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    const appStore = useApp();

    recentShowsStore.shows = [
      {
        displayName: "Concert Tour 2026",
        path: "/shows/concert.ldmx",
        updatedAt: timestamp,
        available: true,
      },
    ];

    render(RecentShowsList);

    await user.click(
      screen.getByRole("button", { name: "Remove Concert Tour 2026 from recent shows" }),
    );

    expect(recentShowsStore.remove).toHaveBeenCalledWith("/shows/concert.ldmx");
    expect(appStore.loadShow).not.toHaveBeenCalled();
  });

  it("should disable the clean button when every show is available", async () => {
    const recentShowsStore = useRecentShows();
    recentShowsStore.shows = [
      {
        displayName: "Concert Tour 2026",
        path: "/shows/concert.ldmx",
        updatedAt: timestamp,
        available: true,
      },
    ];

    render(RecentShowsList);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Refresh shows" })).toBeEnabled(),
    );
    expect(screen.getByRole("button", { name: "Clean unavailable shows" })).toBeDisabled();
  });

  it("should clean unavailable shows without opening or removing them individually", async () => {
    const user = userEvent.setup();
    const recentShowsStore = useRecentShows();
    const appStore = useApp();

    recentShowsStore.shows = [
      {
        displayName: "Concert Tour 2026",
        path: "/shows/concert.ldmx",
        updatedAt: timestamp,
        available: true,
      },
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: timestamp,
        available: false,
      },
    ];

    render(RecentShowsList);

    const cleanButton = screen.getByRole("button", { name: "Clean unavailable shows" });
    expect(cleanButton).toBeEnabled();

    await user.click(cleanButton);

    expect(recentShowsStore.clean).toHaveBeenCalled();
    expect(recentShowsStore.remove).not.toHaveBeenCalled();
    expect(appStore.loadShow).not.toHaveBeenCalled();
  });

  it("should render the empty state when there are no recent shows", () => {
    render(RecentShowsList);

    expect(screen.getByText("No shows found")).toBeInTheDocument();
  });

  it("should render formatted date without 'Invalid date' when updatedAt is a timestamp string", () => {
    const recentShowsStore = useRecentShows();

    recentShowsStore.shows = [
      {
        displayName: "Festival 2026",
        path: "/shows/festival.ldmx",
        updatedAt: timestamp,
        available: true,
      },
    ];

    render(RecentShowsList);

    expect(screen.getByText("Festival 2026")).toBeInTheDocument();
    expect(screen.queryByText(/invalid/i)).not.toBeInTheDocument();
  });
});
