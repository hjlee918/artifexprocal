import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SessionTable } from "../history/SessionTable";
import { SessionSummaryDto } from "../../bindings";

function makeSession(id: string, name: string, overrides?: Partial<SessionSummaryDto>): SessionSummaryDto {
  return {
    id,
    name,
    created_at: 1714320000000,
    ended_at: null,
    state: "finished",
    target_space: "BT.709",
    tier: "GrayscaleOnly",
    patch_count: 21,
    gamma: 2.4,
    max_de: 1.23,
    avg_de: 0.45,
    ...overrides,
  };
}

describe("SessionTable", () => {
  it("renders session rows", () => {
    const sessions = [
      makeSession("a", "Session A"),
      makeSession("b", "Session B"),
    ];

    render(
      <SessionTable
        sessions={sessions}
        total={2}
        page={0}
        perPage={10}
        onPageChange={vi.fn()}
        onView={vi.fn()}
        onCompare={vi.fn()}
      />
    );

    expect(screen.getByText("Session A")).toBeInTheDocument();
    expect(screen.getByText("Session B")).toBeInTheDocument();
  });

  it("calls onView when View clicked", () => {
    const onView = vi.fn();
    const sessions = [makeSession("a", "Session A")];

    render(
      <SessionTable
        sessions={sessions}
        total={1}
        page={0}
        perPage={10}
        onPageChange={vi.fn()}
        onView={onView}
        onCompare={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText("View"));
    expect(onView).toHaveBeenCalledWith("a");
  });

  it("paginates correctly", () => {
    const onPageChange = vi.fn();
    const sessions = [makeSession("a", "Session A")];

    render(
      <SessionTable
        sessions={sessions}
        total={15}
        page={0}
        perPage={10}
        onPageChange={onPageChange}
        onView={vi.fn()}
        onCompare={vi.fn()}
      />
    );

    expect(screen.getByText(/Page 1 of 2/)).toBeInTheDocument();
    const nextBtn = screen.getByRole("button", { name: "Next" });
    fireEvent.click(nextBtn);
    expect(onPageChange).toHaveBeenCalledWith(1);
  });
});
