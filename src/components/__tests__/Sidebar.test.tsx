import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { Sidebar } from "../Sidebar";

describe("Sidebar", () => {
  it("renders all navigation items", () => {
    render(
      <BrowserRouter>
        <Sidebar />
      </BrowserRouter>
    );
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Calibrate")).toBeInTheDocument();
    expect(screen.getByText("Devices")).toBeInTheDocument();
    expect(screen.getByText("History")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("toggles collapse state", () => {
    render(
      <BrowserRouter>
        <Sidebar />
      </BrowserRouter>
    );
    const sidebar = screen.getByText("ArtifexProCal").closest("aside");
    const toggleBtn = screen.getAllByRole("button")[0];
    fireEvent.click(toggleBtn);
    expect(sidebar).toHaveStyle("width: 64px");
  });
});
